use std::collections::HashMap;
use std::mem::{size_of_val, take};
use std::sync::Arc;
use anyhow::bail;
use anyhow::Result;
use ash::vk::{Buffer, BufferUsageFlags, DeviceSize};
use gpu::BlockHeapConfiguration;
use gpu::BlockHeapStatistics;
use gpu::BufferRange;
use gpu::ResourceFactories;
use gpu_allocator::MemoryLocation;
use index_allocator::FrameIndex;
use crate::resource_scope::buffer_resource_entry::BufferResourceEntry;
use index_allocator::ResourceLimits;
use crate::virtual_buffer::dynamic_buffer_memory::DynamicBufferMemory;
use crate::virtual_buffer::dynamic_heap::DynamicHeap;
use crate::virtual_buffer::physical_buffer::PhysicalBuffer;
use crate::virtual_buffer::virtual_buffer::VirtualBuffer;

pub struct BufferResourceScope {
    resource_factories: Arc<ResourceFactories>,

    buffer_entries: HashMap<VirtualBuffer, BufferResourceEntry>,
    buffer_handles: HashMap<&'static str, VirtualBuffer>,
    next_buffer_id: u32,

    upload_heap: DynamicHeap,
    device_heap: DynamicHeap,

    pending_clears: Vec<BufferRange>,
}

impl BufferResourceScope {
    pub fn create(
        resource_factories: Arc<ResourceFactories>,
        limits: ResourceLimits,
        frame_count: u32,
        ray_tracing: bool,
    ) -> Result<Self> {
        let upload_heap = DynamicHeap::create(BlockHeapConfiguration {
            name: "upload_heap",
            block_size: limits.upload_heap_block_size as DeviceSize,
            usage: BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::TRANSFER_DST
                | BufferUsageFlags::TRANSFER_SRC
                | BufferUsageFlags::VERTEX_BUFFER
                | BufferUsageFlags::INDEX_BUFFER,
            location: MemoryLocation::CpuToGpu,
            frame_count,
        })?;

        let mut device_usage = BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::TRANSFER_DST
            | BufferUsageFlags::INDIRECT_BUFFER;

        if ray_tracing {
            device_usage |= BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR;
        }

        let device_heap = DynamicHeap::create(BlockHeapConfiguration {
            name: "device_heap",
            block_size: limits.device_heap_block_size as DeviceSize,
            usage: device_usage,
            location: MemoryLocation::GpuOnly,
            frame_count,
        })?;

        Ok(Self {
            resource_factories,

            buffer_entries: HashMap::new(),
            buffer_handles: HashMap::new(),

            next_buffer_id: 0,

            upload_heap,
            device_heap,

            pending_clears: Vec::new(),
        })
    }

    pub fn create_dynamic_buffer(
        &mut self,
        label: &'static str,
        memory: DynamicBufferMemory,
        alignment: DeviceSize,
        clear: bool,
    ) -> VirtualBuffer {
        let handle = self.handle(label, |entry| matches!(entry, BufferResourceEntry::Dynamic { .. }));
        self.buffer_entries.insert(handle, BufferResourceEntry::dynamic(label, memory, alignment, clear));

        handle
    }

    pub fn import_buffer(&mut self, buffer_range: BufferRange) -> VirtualBuffer {
        let handle = self.handle(buffer_range.label, |entry| matches!(entry, BufferResourceEntry::Imported { .. }));
        self.buffer_entries.insert(handle, BufferResourceEntry::imported(buffer_range));

        handle
    }

    fn handle(&mut self, label: &'static str, expected: impl Fn(&BufferResourceEntry) -> bool) -> VirtualBuffer {
        if let Some(&handle) = self.buffer_handles.get(label) {
            assert!(
                self.buffer_entries.get(&handle).is_none_or(expected),
                "Buffer label '{label}' is already declared with a different kind",
            );

            return handle;
        }

        let handle = VirtualBuffer::new(self.next_buffer_id);
        self.next_buffer_id += 1;

        self.buffer_handles.insert(label, handle);

        handle
    }

    pub fn begin_frame(&mut self, frame_index: FrameIndex) -> Result<()> {
        let buffer_factory = &self.resource_factories.buffer_factory;

        self.upload_heap.begin_frame(buffer_factory, frame_index)?;
        self.device_heap.begin_frame(buffer_factory, frame_index)?;

        Ok(())
    }

    pub fn bind_dynamic_slice<T>(&mut self, handle: VirtualBuffer, data: &[T]) -> Result<()> {
        let (label, alignment) = self.dynamic(handle, DynamicBufferMemory::Upload)?;
        let buffer_factory = &self.resource_factories.buffer_factory;

        let range = self.upload_heap.upload(
            buffer_factory,
            handle,
            label,
            size_of_val(data) as DeviceSize,
            alignment,
        )?;

        range.write(data)
    }

    pub fn bind_dynamic_region(&mut self, handle: VirtualBuffer, size: DeviceSize) -> Result<()> {
        let (label, alignment) = self.dynamic(handle, DynamicBufferMemory::Device)?;
        let clear = self.dynamic_clear(handle);
        let reserved = self.device_heap.binding(handle).is_some();
        let buffer_factory = &self.resource_factories.buffer_factory;

        let range = self.device_heap.reserve(buffer_factory, handle, label, size, alignment)?;

        if clear && !reserved {
            self.pending_clears.push(range);
        }

        Ok(())
    }

    pub fn take_pending_clears(&mut self) -> Vec<BufferRange> {
        take(&mut self.pending_clears)
    }

    pub fn ensure_bound(&self, handle: VirtualBuffer) -> Result<()> {
        let Some(entry) = self.buffer_entries.get(&handle) else {
            bail!("Unknown VirtualBuffer handle {}", handle.handle)
        };

        let BufferResourceEntry::Dynamic { label, memory, .. } = entry else {
            return Ok(());
        };

        if self.dynamic_heap(*memory).binding(handle).is_none() {
            bail!("Dynamic buffer '{label}' is read without being written this frame")
        }

        Ok(())
    }

    pub fn get_physical_buffer(&self, handle: VirtualBuffer) -> PhysicalBuffer {
        let entry = self.buffer_entries.get(&handle).expect("Unknown VirtualBuffer handle");

        match entry {
            BufferResourceEntry::Imported { range } => PhysicalBuffer::create(*range),
            BufferResourceEntry::Dynamic { label, memory, .. } => {
                match self.dynamic_heap(*memory).binding(handle) {
                    Some(range) => PhysicalBuffer::create(range),
                    None => panic!("Dynamic buffer '{label}' is resolved without being written this frame"),
                }
            }
        }
    }

    pub fn heap_buffers(&self) -> Vec<Buffer> {
        self.upload_heap.buffers()
            .chain(self.device_heap.buffers())
            .collect()
    }

    pub fn upload_heap_statistics(&self) -> BlockHeapStatistics {
        self.upload_heap.statistics()
    }

    pub fn device_heap_statistics(&self) -> BlockHeapStatistics {
        self.device_heap.statistics()
    }

    pub fn destroy(self) -> Result<()> {
        let resource_factories = self.resource_factories.clone();
        let buffer_factory = &resource_factories.buffer_factory;

        self.upload_heap.destroy(buffer_factory)?;
        self.device_heap.destroy(buffer_factory)?;

        Ok(())
    }

    fn dynamic(
        &self,
        handle: VirtualBuffer,
        memory: DynamicBufferMemory,
    ) -> Result<(&'static str, DeviceSize)> {
        let Some(entry) = self.buffer_entries.get(&handle) else {
            bail!("Unknown VirtualBuffer handle {}", handle.handle)
        };

        let BufferResourceEntry::Dynamic { label, memory: entry_memory, alignment, .. } = entry else {
            bail!("Buffer '{}' is not dynamic", entry.label())
        };

        if *entry_memory != memory {
            bail!("Dynamic buffer '{label}' is {:?}, written as {:?}", entry_memory, memory)
        }

        Ok((*label, *alignment))
    }

    fn dynamic_clear(&self, handle: VirtualBuffer) -> bool {
        matches!(
            self.buffer_entries.get(&handle),
            Some(BufferResourceEntry::Dynamic { clear: true, .. })
        )
    }

    fn dynamic_heap(&self, memory: DynamicBufferMemory) -> &DynamicHeap {
        match memory {
            DynamicBufferMemory::Upload => &self.upload_heap,
            DynamicBufferMemory::Device => &self.device_heap,
        }
    }
}
