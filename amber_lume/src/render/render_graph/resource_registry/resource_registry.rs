use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::render_graph::resource_registry::resource_entry::ResourceEntry;
use crate::render::render_graph::virtual_image::image_blueprint::ImageBlueprint;
use crate::render::render_graph::virtual_image::physical_image::PhysicalImage;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use anyhow::Result;
use ash::vk::{Extent2D, Extent3D, Image, ImageSubresourceRange, ImageTiling, ImageType, ImageView, SampleCountFlags, SharingMode};
use std::collections::HashMap;
use std::sync::Arc;
use crate::render::factories::image::image_description::ImageDescription;
use crate::render::render_graph::virtual_image::image_size::ImageSize;

pub struct ResourceRegistry {
    entries: HashMap<VirtualImage, ResourceEntry>,

    next_id: u32,

    resource_factories: Arc<ResourceFactories>,
}

impl ResourceRegistry {
    pub fn new(resource_factories: Arc<ResourceFactories>) -> Self {
        Self {
            entries: HashMap::new(),

            next_id: 0,

            resource_factories,
        }
    }

    pub fn create_image(&mut self, label: &'static str, blueprint: ImageBlueprint) -> VirtualImage {
        let handle = VirtualImage::new(self.next_id);

        self.next_id += 1;
        self.entries.insert(handle, ResourceEntry::transient(label, blueprint));

        handle
    }

    pub fn import_image(
        &mut self,
        image: Image,
        image_view: ImageView,
        layers: Vec<ImageView>,
        extent: Extent2D,
        subresource_range: ImageSubresourceRange,
    ) -> VirtualImage {
        let handle = VirtualImage::new(self.next_id);

        self.next_id += 1;
        self.entries.insert(
            handle,
            ResourceEntry::imported(image, image_view, layers, extent, subresource_range),
        );
        handle
    }

    pub fn import_image_placeholder(&mut self) -> VirtualImage {
        self.import_image(
            Image::null(),
            ImageView::null(),
            Vec::new(),
            Extent2D::default(),
            ImageSubresourceRange::default(),
        )
    }

    pub fn update_imported(
        &mut self,
        handle: VirtualImage,
        image: Image,
        image_view: ImageView,
        layers: Vec<ImageView>,
        extent: Extent2D,
        subresource_range: ImageSubresourceRange,
    ) {
        self.entries.insert(
            handle,
            ResourceEntry::imported(image, image_view, layers, extent, subresource_range),
        );
    }

    pub fn build(
        &mut self,
        swapchain_extent: Extent2D,
    ) -> Result<()> {
        for entry in self.entries.values_mut() {
            if let ResourceEntry::Transient { label, blueprint, managed } = entry {
                if let Some(old) = managed.take() {
                    self.resource_factories.managed_image_factory.destroy_image(old)?;
                }

                let (width, height) = match blueprint.size {
                    ImageSize::FullResolution => (swapchain_extent.width, swapchain_extent.height),
                    ImageSize::Absolute { width, height } => (width, height),
                };

                let image_description = ImageDescription {
                    image_type: ImageType::TYPE_2D,
                    format: blueprint.format,
                    extent: Extent3D {
                        width,
                        height,
                        depth: 1,
                    },
                    mip_levels: 1,
                    array_layers: 1,
                    samples: SampleCountFlags::TYPE_1,
                    tiling: ImageTiling::OPTIMAL,
                    usage: blueprint.usage,
                    sharing_mode: SharingMode::EXCLUSIVE,
                };

                *managed = Some(self.resource_factories.managed_image_factory.allocate(
                    label,
                    image_description,
                    blueprint.image_view_description,
                )?)
            }
        }

        Ok(())
    }

    pub fn get(&self, handle: VirtualImage) -> PhysicalImage {
        let entry = self.entries.get(&handle).expect("Unknown VirtualImage handle");

        match entry {
            ResourceEntry::Transient { managed, .. } => {
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
                    subresource_range: managed.image_subresource_range,
                }
            }
            ResourceEntry::Imported {
                image,
                image_view,
                layers,
                extent,
                subresource_range,
            } => {
                PhysicalImage {
                    image: *image,
                    image_view: *image_view,
                    extent: *extent,
                    layers: layers.clone(),
                    subresource_range: *subresource_range,
                }
            }
        }
    }

    pub fn destroy(&mut self) -> Result<()> {
        for entry in self.entries.values_mut() {
            if let ResourceEntry::Transient { managed, .. } = entry {
                if let Some(managed) = managed.take() {
                    self.resource_factories.managed_image_factory.destroy_image(managed)?;
                }
            }
        }

        Ok(())
    }
}
