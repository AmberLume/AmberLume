use index_allocator::FrameIndex;
use gpu::ManagedBufferFactory;
use crate::render::resource_scope::buffer_resource_entry::BufferResourceEntry;
use crate::render::render_graph::virtual_buffer::buffer_blueprint::BufferBlueprint;
use crate::render::render_graph::virtual_buffer::physical_buffer::PhysicalBuffer;
use crate::render::render_graph::virtual_buffer::transient_buffer_heap::{align_up, TransientBufferHeap, TRANSIENT_BUFFER_ALIGNMENT};
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use anyhow::Result;
use ash::vk::{Buffer, BufferUsageFlags, DeviceAddress, DeviceSize};
use std::collections::HashMap;

pub struct BufferResourceScope {
    pub buffer_entries: HashMap<VirtualBuffer, BufferResourceEntry>,
    buffer_handles: HashMap<&'static str, VirtualBuffer>,
    next_buffer_id: u32,

    transient_buffer_heap: Option<TransientBufferHeap>,
}

impl BufferResourceScope {
    pub fn new() -> Self {
        Self {
            buffer_entries: HashMap::new(),
            buffer_handles: HashMap::new(),
            next_buffer_id: 0,

            transient_buffer_heap: None,
        }
    }

    pub fn create_buffer(&mut self, label: &'static str, blueprint: BufferBlueprint) -> VirtualBuffer {
        if let Some(&handle) = self.buffer_handles.get(label) {
            let matches = matches!(
                self.buffer_entries.get(&handle),
                Some(BufferResourceEntry::Transient { blueprint: existing, .. }) if *existing == blueprint
            );

            if !matches {
                self.buffer_entries.insert(handle, BufferResourceEntry::transient(label, blueprint));
            }

            return handle;
        }

        let handle = VirtualBuffer::new(self.next_buffer_id);
        self.next_buffer_id += 1;

        self.buffer_handles.insert(label, handle);
        self.buffer_entries.insert(handle, BufferResourceEntry::transient(label, blueprint));

        handle
    }

    pub fn build_transient_buffers(
        &mut self,
        buffer_factory: &ManagedBufferFactory,
        frame_count: u32,
        lifetimes: &HashMap<VirtualBuffer, (usize, usize)>,
    ) -> Result<()> {
        let mut transients: Vec<(VirtualBuffer, BufferBlueprint)> = self.buffer_entries.iter()
            .filter_map(|(&handle, entry)| match entry {
                BufferResourceEntry::Transient { blueprint, .. } => Some((handle, *blueprint)),
                _ => None,
            })
            .collect();
        transients.sort_by_key(|(handle, _)| handle.handle);

        if transients.is_empty() {
            if let Some(heap) = self.transient_buffer_heap.take() {
                heap.destroy(buffer_factory)?;
            }
            return Ok(());
        }

        let lifetime_of = |handle: &VirtualBuffer| {
            lifetimes.get(handle).copied().unwrap_or((0, usize::MAX))
        };

        let mut order: Vec<usize> = (0..transients.len()).collect();
        order.sort_by(|&a, &b| {
            transients[b].1.size.cmp(&transients[a].1.size)
                .then(transients[a].0.handle.cmp(&transients[b].0.handle))
        });

        let mut usage = BufferUsageFlags::empty();
        let mut placed: Vec<(usize, DeviceSize)> = Vec::with_capacity(transients.len());
        let mut placements: Vec<(VirtualBuffer, DeviceSize)> = Vec::with_capacity(transients.len());
        let mut capacity_per_frame: DeviceSize = 0;

        for &index in &order {
            let (handle, blueprint) = transients[index];
            usage |= blueprint.usage;

            let (start, end) = lifetime_of(&handle);

            let mut forbidden: Vec<(DeviceSize, DeviceSize)> = placed.iter()
                .filter(|(other_index, _)| {
                    let (other_start, other_end) = lifetime_of(&transients[*other_index].0);
                    start <= other_end && other_start <= end
                })
                .map(|(other_index, other_offset)| {
                    (*other_offset, *other_offset + transients[*other_index].1.size)
                })
                .collect();
            forbidden.sort_by_key(|(offset, _)| *offset);

            let mut base_offset: DeviceSize = 0;
            for (occupied_offset, occupied_end) in &forbidden {
                if base_offset + blueprint.size <= *occupied_offset {
                    break;
                }
                base_offset = base_offset.max(align_up(*occupied_end, TRANSIENT_BUFFER_ALIGNMENT));
            }

            placed.push((index, base_offset));
            placements.push((handle, base_offset));
            capacity_per_frame = capacity_per_frame.max(base_offset + blueprint.size);
        }

        let capacity_per_frame = align_up(capacity_per_frame, TRANSIENT_BUFFER_ALIGNMENT);

        let needs_rebuild = !matches!(
            &self.transient_buffer_heap,
            Some(heap) if heap.matches(capacity_per_frame, usage, frame_count)
        );

        if needs_rebuild {
            if let Some(heap) = self.transient_buffer_heap.take() {
                heap.destroy(buffer_factory)?;
            }
            self.transient_buffer_heap = Some(TransientBufferHeap::create(
                buffer_factory,
                capacity_per_frame,
                usage,
                frame_count,
            )?);
        }

        for (handle, base_offset) in placements {
            if let Some(BufferResourceEntry::Transient { base_offset: slot, .. }) = self.buffer_entries.get_mut(&handle) {
                *slot = Some(base_offset);
            }
        }

        Ok(())
    }

    pub fn begin_transient_buffers_frame(&mut self, frame_index: FrameIndex) {
        if let Some(heap) = self.transient_buffer_heap.as_mut() {
            heap.begin_frame(frame_index);
        }
    }

    pub fn import_buffer(
        &mut self,
        label: &'static str,
        buffer: Buffer,
        offset: DeviceSize,
        size: DeviceSize,
        device_address: DeviceAddress,
        mapped_ptr: *mut u8,
    ) -> VirtualBuffer {
        let entry = BufferResourceEntry::imported(buffer, offset, size, device_address, mapped_ptr);

        if let Some(&handle) = self.buffer_handles.get(label) {
            self.buffer_entries.insert(handle, entry);

            return handle;
        }

        let handle = VirtualBuffer::new(self.next_buffer_id);
        self.next_buffer_id += 1;

        self.buffer_handles.insert(label, handle);
        self.buffer_entries.insert(handle, entry);

        handle
    }

    pub fn import_buffer_placeholder(&mut self, label: &'static str) -> VirtualBuffer {
        self.import_buffer(
            label,
            Buffer::null(),
            DeviceSize::default(),
            DeviceSize::default(),
            DeviceAddress::default(),
            Default::default(),
        )
    }

    pub fn rebind_buffer(
        &mut self,
        handle: VirtualBuffer,
        buffer: Buffer,
        offset: DeviceSize,
        size: DeviceSize,
        device_address: DeviceAddress,
        mapped_ptr: *mut u8,
    ) {
        self.buffer_entries.insert(
            handle,
            BufferResourceEntry::imported(buffer, offset, size, device_address, mapped_ptr),
        );
    }

    pub fn get_physical_buffer(&self, handle: VirtualBuffer) -> PhysicalBuffer {
        let entry = self.buffer_entries.get(&handle).expect("Unknown VirtualBuffer handle");

        match entry {
            BufferResourceEntry::Transient { blueprint, base_offset, .. } => {
                let base_offset = base_offset
                    .expect("Transient buffer not placed — call build() before execute()");
                let heap = self.transient_buffer_heap.as_ref()
                    .expect("Transient buffer heap not built");

                heap.physical(base_offset, blueprint.size)
            }
            BufferResourceEntry::Imported {
                buffer,
                offset,
                size,
                device_address,
                mapped_ptr,
            } => PhysicalBuffer::create(*buffer, *offset, *size, *device_address, *mapped_ptr),
        }
    }

    pub fn destroy(self, buffer_factory: &ManagedBufferFactory) -> Result<()> {
        if let Some(heap) = self.transient_buffer_heap {
            heap.destroy(buffer_factory)?;
        }

        Ok(())
    }
}
