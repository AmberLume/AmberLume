use crate::render::render_graph::resource_registry::image_resource_entry::ImageResourceEntry;
use crate::render::render_graph::virtual_image::image_blueprint::ImageBlueprint;
use crate::render::render_graph::virtual_image::physical_image::PhysicalImage;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use anyhow::Result;
use ash::vk::{Buffer, DeviceAddress, DeviceSize, Extent2D, Extent3D, Format, Image, ImageSubresourceRange, ImageTiling, ImageType, ImageView, SampleCountFlags, SharingMode};
use std::collections::HashMap;
use std::sync::Arc;
use crate::render::factories::image::image_description::ImageDescription;
use crate::render::factories::image::managed_image_factory::ManagedImageFactory;
use crate::render::render_graph::resource_registry::buffer_resource_entry::BufferResourceEntry;
use crate::render::render_graph::virtual_buffer::physical_buffer::PhysicalBuffer;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::resources::store::providers::image::image_backend::ImageBackend;
use crate::resources::store::providers::image::image_config::ImageConfig;
use crate::resources::store::providers::resource_provider::{ResourceId, ResourceProvider};
use crate::utils::arc_utils::ArcUnwrapOrErr;

pub struct ResourceRegistry {
    image_entries: HashMap<VirtualImage, ImageResourceEntry>,
    next_image_id: u32,

    buffer_entries: HashMap<VirtualBuffer, BufferResourceEntry>,
    next_buffer_id: u32,
}

impl ResourceRegistry {
    pub fn new() -> Self {
        Self {
            image_entries: HashMap::new(),
            next_image_id: 0,

            buffer_entries: HashMap::new(),
            next_buffer_id: 0,
        }
    }

    pub fn create_image(&mut self, label: &'static str, blueprint: ImageBlueprint) -> VirtualImage {
        let handle = VirtualImage::new(self.next_image_id);

        self.next_image_id += 1;
        self.image_entries.insert(handle, ImageResourceEntry::transient(label, blueprint));

        handle
    }

    pub fn import_image(
        &mut self,
        image: Image,
        image_view: ImageView,
        layers: Vec<ImageView>,
        extent: Extent2D,
        format: Format,
        subresource_range: ImageSubresourceRange,
        descriptor_id: Option<ResourceId>,
    ) -> VirtualImage {
        let handle = VirtualImage::new(self.next_image_id);
        self.next_image_id += 1;

        self.image_entries.insert(
            handle,
            ImageResourceEntry::imported(image, image_view, layers, extent, format, subresource_range, descriptor_id),
        );

        handle
    }

    pub fn import_buffer(
        &mut self,
        buffer: Buffer,
        offset: DeviceSize,
        size: DeviceSize,
        device_address: DeviceAddress,
        mapped_ptr: *mut u8,
    ) -> VirtualBuffer {
        let handle = VirtualBuffer::new(self.next_buffer_id);
        self.next_buffer_id += 1;

        self.buffer_entries.insert(
            handle,
            BufferResourceEntry::imported(buffer, offset, size, device_address, mapped_ptr),
        );

        handle
    }

    pub fn import_image_placeholder(&mut self) -> VirtualImage {
        self.import_image(
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
            ImageResourceEntry::imported(image, image_view, layers, extent, format, subresource_range, None),
        );
    }

    pub fn import_buffer_placeholder(&mut self) -> VirtualBuffer {
        self.import_buffer(
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

    pub fn build(
        &mut self,
        swapchain_extent: Extent2D,
        image_factory: &ManagedImageFactory,
        image_provider: &ResourceProvider<ImageBackend>,
    ) -> Result<()> {
        for entry in self.image_entries.values_mut() {
            if let ImageResourceEntry::Transient { label, blueprint, res_ref, managed } = entry {
                if res_ref.is_none() {
                    if let Some(old) = managed.take() {
                        image_factory.destroy_image(old.try_unwrap()?)?;
                    }
                }

                let extent = blueprint.size.resolve(swapchain_extent);

                let image_description = ImageDescription {
                    image_type: ImageType::TYPE_2D,
                    format: blueprint.format,
                    extent: Extent3D {
                        width: extent.width,
                        height: extent.height,
                        depth: 1,
                    },
                    mip_levels: 1,
                    array_layers: 1,
                    samples: SampleCountFlags::TYPE_1,
                    tiling: ImageTiling::OPTIMAL,
                    usage: blueprint.usage,
                    sharing_mode: SharingMode::EXCLUSIVE,
                };

                if let Some((binding, sampler_type)) = blueprint.descriptor {
                    let new_res_ref = image_provider.acquire_sync(ImageConfig::Inbuilt {
                        label: label.to_string(),
                        image_description,
                        image_view_description: blueprint.image_view_description.clone(),
                        binding,
                        sampler_type,
                        data: None,
                    });

                    let new_managed = image_provider
                        .get_resource(new_res_ref.id)
                        .expect("Image must be available after acquire");

                    *managed = Some(new_managed);
                    *res_ref = Some(new_res_ref);
                } else {
                    let managed_image = image_factory.allocate(
                        label,
                        image_description,
                        blueprint.image_view_description,
                    )?;
                    *managed = Some(Arc::new(managed_image));
                }
            }
        }

        Ok(())
    }

    pub fn get_physical_image(&self, handle: VirtualImage) -> PhysicalImage {
        let entry = self.image_entries.get(&handle).expect("Unknown VirtualImage handle");

        match entry {
            ImageResourceEntry::Transient { managed, res_ref, .. } => {
                let managed = managed.as_ref()
                    .expect("Transient image not built — call build() before execute()");

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
            } => {
                PhysicalImage {
                    image: *image,
                    image_view: *image_view,
                    layers: layers.clone(),
                    extent: *extent,
                    format: *format,
                    subresource_range: *subresource_range,
                    descriptor_id: *descriptor_id,
                }
            }
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
            } => {
                PhysicalBuffer::create(
                    *buffer,
                    *offset,
                    *size,
                    *device_address,
                    *mapped_ptr,
                )
            }
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
