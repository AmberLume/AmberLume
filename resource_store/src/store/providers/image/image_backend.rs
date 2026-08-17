use anyhow::Result;
use parking_lot::Mutex;
use ash::vk::{Extent3D, ImageView, ImageAspectFlags, ImageCreateFlags, ImageSubresourceLayers, ImageTiling, ImageType, ImageUsageFlags, ImageViewType, SampleCountFlags, SharingMode};
use std::sync::Arc;
use tracing::info;
use asset_codec::TextureData;
use crate::store::providers::image::texture_format::TextureFormat;
use gpu::ImageDescription;
use gpu::ImageViewDescription;
use gpu::ManagedImage;
use gpu::ResourceTransfer;
use resource_residency::ResourceBackend;
use index_allocator::ResourceId;
use gpu::ResourceFactories;
use gpu::ManagedDescriptorSet;
use resource_reader::ResourceReader;
use crate::store::providers::image::image_config::ImageConfig;

pub struct ImageBackend {
    texture_format: TextureFormat,

    resource_factories: Arc<ResourceFactories>,
    resource_reader: Arc<dyn ResourceReader>,

    descriptor_set: ManagedDescriptorSet,

    resource_transfer: Arc<ResourceTransfer>,

    default_image_view: Mutex<Option<ImageView>>,
}

impl ImageBackend {
    pub(crate) fn new(
        texture_format: TextureFormat,
        resource_factories: Arc<ResourceFactories>,
        resource_reader: Arc<dyn ResourceReader>,
        descriptor_set: ManagedDescriptorSet,
        resource_transfer: Arc<ResourceTransfer>,
    ) -> Self {
        Self {
            texture_format,

            resource_factories,
            resource_reader,

            descriptor_set,

            resource_transfer,

            default_image_view: Mutex::new(None),
        }
    }

    pub(crate) fn set_default_image_view(&self, image_view: ImageView) {
        *self.default_image_view.lock() = Some(image_view);
    }
}

impl ResourceBackend for ImageBackend {
    type Config = ImageConfig;
    type Output = ManagedImage;
    type Statistics = ();

    fn create(
        &self,
        id: &ResourceId,
        config: Self::Config,
    ) -> Result<Self::Output> {
        let managed_image = match config {
            ImageConfig::Alpaca { resource_key } => {
                let image_bytes = self.resource_reader.get_resource(&resource_key)?;

                let texture_data = TextureData::decode(
                    image_bytes,
                    self.texture_format.block_format,
                )?;

                let width = texture_data.width;
                let height = texture_data.height;

                let mip_levels = texture_data.mip_levels;

                let format = if texture_data.is_srgb {
                    self.texture_format.color_srgb
                } else {
                    self.texture_format.linear
                };

                let image_description = ImageDescription {
                    image_type: ImageType::TYPE_2D,
                    format,
                    extent: Extent3D {
                        width,
                        height,
                        depth: 1,
                    },
                    mip_levels,
                    array_layers: 1,
                    samples: SampleCountFlags::TYPE_1,
                    tiling: ImageTiling::OPTIMAL,
                    usage: ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
                    sharing_mode: SharingMode::EXCLUSIVE,
                    flags: ImageCreateFlags::empty(),
                };
                let image_view_description = ImageViewDescription {
                    image_view_type: ImageViewType::TYPE_2D,
                    image_aspect_flags: ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: mip_levels,
                    base_array_layer: 0,
                    layer_count: 1,
                };

                let managed_image = self.resource_factories.managed_image_factory.allocate(
                    &resource_key,
                    image_description,
                    image_view_description,
                )?;

                for (index, level_data) in texture_data.levels.iter().enumerate() {
                    let (level_width, level_height) = texture_data.level_extent(index as u32);

                    self.resource_transfer.load_image(
                        managed_image.image,
                        Extent3D {
                            width: level_width,
                            height: level_height,
                            depth: 1,
                        },
                        ImageSubresourceLayers {
                            aspect_mask: ImageAspectFlags::COLOR,
                            mip_level: index as u32,
                            base_array_layer: 0,
                            layer_count: 1,
                        },
                        managed_image.image_description.mip_levels,
                        1,
                        &level_data,
                    )?;
                }

                self.descriptor_set.write(*id, managed_image.image_view);

                managed_image
            }
            ImageConfig::Inbuilt {
                label,

                image_description,
                image_view_description,

                data,
            } => {
                let extent = image_description.extent;
                let managed_image = self.resource_factories.managed_image_factory.allocate(
                    &label,
                    image_description,
                    image_view_description,
                )?;

                if let Some(data) = data {
                    self.resource_transfer.load_image(
                        managed_image.image,
                        extent,
                        ImageSubresourceLayers {
                            aspect_mask: ImageAspectFlags::COLOR,
                            mip_level: 0,
                            base_array_layer: 0,
                            layer_count: 1,
                        },
                        managed_image.image_description.mip_levels,
                        1,
                        &data,
                    )?;
                }

                self.descriptor_set.write(*id, managed_image.image_view);

                managed_image
            }
        };

        info!("Image resource {} is now available", id.inner);

        Ok(managed_image)
    }

    fn erase(&self, id: &ResourceId) -> Result<()> {
        if let Some(image_view) = *self.default_image_view.lock() {
            self.descriptor_set.write(*id, image_view);
        }

        Ok(())
    }

    fn statistics(&self) -> Self::Statistics {
        ()
    }
    
    fn destroy_resource(&self, resource: Self::Output) -> Result<()> {
        self.resource_factories.managed_image_factory.destroy_image(resource)?;

        Ok(())
    }
}
