use crate::render::render_graph::resource_registry::resource_entry::ResourceEntry;
use crate::render::render_graph::virtual_image::image_blueprint::ImageBlueprint;
use crate::render::render_graph::virtual_image::physical_image::PhysicalImage;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use anyhow::Result;
use ash::vk::{Extent2D, Extent3D, Image, ImageSubresourceRange, ImageTiling, ImageType, ImageView, SampleCountFlags, SharingMode};
use std::collections::HashMap;
use std::sync::Arc;
use crate::render::factories::image::image_description::ImageDescription;
use crate::render::factories::image::managed_image_factory::ManagedImageFactory;
use crate::render::render_graph::virtual_image::image_size::ImageSize;
use crate::resources::dynamic::image::image_backend::ImageBackend;
use crate::resources::dynamic::image::image_config::ImageConfig;
use crate::resources::dynamic::resource_provider::{ResourceId, ResourceProvider};
use crate::utils::arc_utils::ArcUnwrapOrErr;

pub struct ResourceRegistry {
    entries: HashMap<VirtualImage, ResourceEntry>,

    next_id: u32,
}

impl ResourceRegistry {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),

            next_id: 0,
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
        descriptor_id: Option<ResourceId>,
    ) -> VirtualImage {
        let handle = VirtualImage::new(self.next_id);

        self.next_id += 1;
        self.entries.insert(
            handle,
            ResourceEntry::imported(image, image_view, layers, extent, subresource_range, descriptor_id),
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
            None,
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
            ResourceEntry::imported(image, image_view, layers, extent, subresource_range, None),
        );
    }

    pub fn build(
        &mut self,
        swapchain_extent: Extent2D,
        image_factory: &ManagedImageFactory,
        image_provider: &ResourceProvider<ImageBackend>,
    ) -> Result<()> {
        for entry in self.entries.values_mut() {
            if let ResourceEntry::Transient { label, blueprint, res_ref, managed } = entry {
                if res_ref.is_none() {
                    if let Some(old) = managed.take() {
                        image_factory.destroy_image(old.try_unwrap()?)?;
                    }
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

    pub fn get(&self, handle: VirtualImage) -> PhysicalImage {
        let entry = self.entries.get(&handle).expect("Unknown VirtualImage handle");

        match entry {
            ResourceEntry::Transient { managed, res_ref, .. } => {
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
                    descriptor_id: res_ref.as_ref().map(|r| r.id),
                }
            }
            ResourceEntry::Imported {
                image,
                image_view,
                layers,
                extent,
                subresource_range,
                descriptor_id,
            } => {
                PhysicalImage {
                    image: *image,
                    image_view: *image_view,
                    extent: *extent,
                    layers: layers.clone(),
                    subresource_range: *subresource_range,
                    descriptor_id: *descriptor_id,
                }
            }
        }
    }

    pub fn destroy(self, image_factory: &ManagedImageFactory) -> Result<()> {
        for entry in self.entries.into_values() {
            if let ResourceEntry::Transient { res_ref, managed, .. } = entry {
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
