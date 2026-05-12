use crate::render::factories::image::managed_image_factory::ManagedImageFactory;
use crate::render::render_graph::resource_registry::buffer_resource_entry::BufferResourceEntry;
use crate::render::render_graph::resource_registry::image_resource_entry::ImageResourceEntry;
use crate::render::render_graph::virtual_buffer::physical_buffer::PhysicalBuffer;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
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
        layers: Vec<ImageView>,
        extent: Extent2D,
        format: Format,
        subresource_range: ImageSubresourceRange,
        descriptor_id: Option<ResourceId>,
    ) -> VirtualImage {
        let entry = ImageResourceEntry::imported(
            image,
            image_view,
            layers,
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
            Vec::new(),
            Extent2D::default(),
            Format::UNDEFINED,
            ImageSubresourceRange::default(),
            None,
        )
    }

    pub fn update_imported_image(
        &mut self,
        handle: VirtualImage,
        image: Image,
        image_view: ImageView,
        layers: Vec<ImageView>,
        format: Format,
        extent: Extent2D,
        subresource_range: ImageSubresourceRange,
    ) {
        self.image_entries.insert(
            handle,
            ImageResourceEntry::imported(
                image,
                image_view,
                layers,
                extent,
                format,
                subresource_range,
                None,
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

    pub fn update_imported_buffer(
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
                    layers: managed.image_view_layers.clone(),
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
                layers,
                extent,
                format,
                subresource_range,
                descriptor_id,
            } => PhysicalImage {
                image: *image,
                image_view: *image_view,
                layers: layers.clone(),
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
            BufferResourceEntry::Imported {
                buffer,
                offset,
                size,
                device_address,
                mapped_ptr,
            } => PhysicalBuffer::create(*buffer, *offset, *size, *device_address, *mapped_ptr),
        }
    }

    pub fn destroy(self, image_factory: &ManagedImageFactory) -> Result<()> {
        for entry in self.image_entries.into_values() {
            if let ImageResourceEntry::Transient { res_ref, managed, .. } = entry {
                if res_ref.is_none() {
                    if let Some(managed) = managed {
                        image_factory.destroy_image(managed.try_unwrap()?)?;
                    }
                }
            }
        }

        Ok(())
    }
}
