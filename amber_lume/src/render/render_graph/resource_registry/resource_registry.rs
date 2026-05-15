use crate::ids::FrameIndex;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::image::managed_image_factory::ManagedImageFactory;
use crate::render::render_graph::resource_registry::buffer_resource_entry::BufferResourceEntry;
use crate::render::render_graph::resource_registry::image_resource_entry::ImageResourceEntry;
use crate::render::render_graph::virtual_buffer::buffer_blueprint::BufferBlueprint;
use crate::render::render_graph::virtual_buffer::physical_buffer::PhysicalBuffer;
use crate::render::render_graph::virtual_buffer::transient_buffer_heap::{align_up, TransientBufferHeap, TRANSIENT_BUFFER_ALIGNMENT};
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use ash::vk::BufferUsageFlags;
use crate::render::render_graph::virtual_image::image_blueprint::ImageBlueprint;
use crate::render::render_graph::virtual_image::physical_image::PhysicalImage;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::resources::store::providers::resource_provider::ResourceId;
use crate::utils::arc_utils::ArcUnwrapOrErr;
use anyhow::Result;
use ash::vk::{
    Buffer, DeviceAddress, DeviceSize, Extent2D, Format, Image, ImageSubresourceRange, ImageView,
};
use std::collections::HashMap;

pub struct ResourceRegistry {
    pub image_entries: HashMap<VirtualImage, ImageResourceEntry>,
    image_handles: HashMap<&'static str, VirtualImage>,
    next_image_id: u32,

    pub buffer_entries: HashMap<VirtualBuffer, BufferResourceEntry>,
    buffer_handles: HashMap<&'static str, VirtualBuffer>,
    next_buffer_id: u32,

    transient_buffer_heap: Option<TransientBufferHeap>,
}

impl ResourceRegistry {
    pub fn new() -> Self {
        Self {
            image_entries: HashMap::new(),
            image_handles: HashMap::new(),
            next_image_id: 0,

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

        let mut head: DeviceSize = 0;
        let mut usage = BufferUsageFlags::empty();
        let mut placements: Vec<(VirtualBuffer, DeviceSize)> = Vec::with_capacity(transients.len());

        for (handle, blueprint) in &transients {
            let base_offset = align_up(head, TRANSIENT_BUFFER_ALIGNMENT);
            head = base_offset + blueprint.size;
            usage |= blueprint.usage;
            placements.push((*handle, base_offset));
        }

        let capacity_per_frame = align_up(head, TRANSIENT_BUFFER_ALIGNMENT);

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

    pub fn create_image(&mut self, label: &'static str, blueprint: ImageBlueprint) -> VirtualImage {
        if let Some(&handle) = self.image_handles.get(label) {
            let matches = matches!(
                self.image_entries.get(&handle),
                Some(ImageResourceEntry::Transient { blueprint: existing, .. }) if *existing == blueprint
            );

            if !matches {
                self.image_entries.insert(handle, ImageResourceEntry::transient(label, blueprint));
            }

            return handle;
        }

        let handle = VirtualImage::new(self.next_image_id);
        self.next_image_id += 1;

        self.image_handles.insert(label, handle);
        self.image_entries.insert(handle, ImageResourceEntry::transient(label, blueprint));

        handle
    }

    pub fn import_image(
        &mut self,
        label: &'static str,
        image: Image,
        image_view: ImageView,
        extent: Extent2D,
        format: Format,
        subresource_range: ImageSubresourceRange,
        descriptor_id: Option<ResourceId>,
    ) -> VirtualImage {
        let entry = ImageResourceEntry::imported(
            image,
            image_view,
            extent,
            format,
            subresource_range,
            descriptor_id,
        );

        if let Some(&handle) = self.image_handles.get(label) {
            self.image_entries.insert(handle, entry);

            return handle;
        }

        let handle = VirtualImage::new(self.next_image_id);
        self.next_image_id += 1;

        self.image_handles.insert(label, handle);
        self.image_entries.insert(handle, entry);

        handle
    }

    pub fn import_image_placeholder(&mut self, label: &'static str) -> VirtualImage {
        self.import_image(
            label,
            Image::null(),
            ImageView::null(),
            Extent2D::default(),
            Format::UNDEFINED,
            ImageSubresourceRange::default(),
            None,
        )
    }

    pub fn rebind_image(
        &mut self,
        handle: VirtualImage,
        image: Image,
        image_view: ImageView,
        extent: Extent2D,
        format: Format,
        subresource_range: ImageSubresourceRange,
        descriptor_id: Option<ResourceId>,
    ) {
        self.image_entries.insert(
            handle,
            ImageResourceEntry::imported(
                image,
                image_view,
                extent,
                format,
                subresource_range,
                descriptor_id,
            ),
        );
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

    pub fn get_physical_image(&self, handle: VirtualImage) -> PhysicalImage {
        let entry = self.image_entries.get(&handle).expect("Unknown VirtualImage handle");

        match entry {
            ImageResourceEntry::Transient {
                managed, res_ref, ..
            } => {
                let managed = managed.as_ref().expect("Transient image not built — call build() before execute()");

                PhysicalImage {
                    image: managed.image,
                    image_view: managed.image_view,
                    extent: Extent2D {
                        width: managed.image_description.extent.width,
                        height: managed.image_description.extent.height,
                    },
                    format: managed.image_description.format,
                    subresource_range: managed.image_subresource_range,
                    descriptor_id: res_ref.as_ref().map(|r| r.id),
                }
            }
            ImageResourceEntry::Imported {
                image,
                image_view,
                extent,
                format,
                subresource_range,
                descriptor_id,
            } => PhysicalImage {
                image: *image,
                image_view: *image_view,
                extent: *extent,
                format: *format,
                subresource_range: *subresource_range,
                descriptor_id: *descriptor_id,
            },
        }
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

    pub fn destroy(
        self,
        image_factory: &ManagedImageFactory,
        buffer_factory: &ManagedBufferFactory,
    ) -> Result<()> {
        for entry in self.image_entries.into_values() {
            if let ImageResourceEntry::Transient { res_ref, managed, .. } = entry {
                if res_ref.is_none() {
                    if let Some(managed) = managed {
                        image_factory.destroy_image(managed.try_unwrap()?)?;
                    }
                }
            }
        }

        if let Some(heap) = self.transient_buffer_heap {
            heap.destroy(buffer_factory)?;
        }

        Ok(())
    }
}
