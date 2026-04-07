use crate::resources::alpaca_resource_reader::AlpacaResourceReader;
use anyhow::{bail, Result};
use ash::vk::{Extent3D, ImageAspectFlags, ImageSubresourceLayers, ImageTiling, ImageType, ImageUsageFlags, ImageViewType, SampleCountFlags, SharingMode};
use ktx2::{Reader, SupercompressionScheme};
use std::sync::Arc;
use tracing::info;
use crate::render::device::texture_format::TextureFormat;
use crate::render::factories::image::image_description::ImageDescription;
use crate::render::factories::image::image_view_description::ImageViewDescription;
use crate::render::factories::image::managed_image::ManagedImage;
use crate::render::resources::resource_loader::ResourceLoader;
use crate::resources::binding_layout::descriptor_set_manager::GlobalDescriptorSetBindings;
use crate::resources::dynamic::image::image_config::ImageConfig;
use crate::resources::dynamic::image::transcode_utils::transcode;
use crate::resources::dynamic::resource_backend::ResourceBackend;
use crate::resources::dynamic::resource_provider::ResourceId;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::resources::binding_layout::binding_layout::BindingLayout;
use crate::resources::sampler_registry::SamplerType;

pub struct ImageBackend {
    texture_format: TextureFormat,

    resource_factories: Arc<ResourceFactories>,
    alpaca_resource_reader: Arc<AlpacaResourceReader>,

    binding_layout: Arc<BindingLayout>,

    resource_loader: Arc<ResourceLoader>,
}

impl ImageBackend {
    pub fn new(
        texture_format: TextureFormat,
        resource_factories: Arc<ResourceFactories>,
        alpaca_resource_reader: Arc<AlpacaResourceReader>,
        binding_layout: Arc<BindingLayout>,
        resource_loader: Arc<ResourceLoader>,
    ) -> Self {
        Self {
            texture_format,

            resource_factories,
            alpaca_resource_reader,

            binding_layout,

            resource_loader,
        }
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
                let image_bytes = self.alpaca_resource_reader.get_resource(&resource_key)?;

                let reader = Reader::new(image_bytes)?;
                let header = reader.header();

                let width = header.pixel_width;
                let height = header.pixel_height;

                let mip_levels = header.level_count;

                if reader.header().supercompression_scheme != Some(SupercompressionScheme::Zstandard) {
                    bail!("Unsupported supercompression scheme: {:?}", reader.header().supercompression_scheme);
                }

                let format = self.texture_format.color_srgb;

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
                };
                let image_view_description = ImageViewDescription {
                    image_view_type: ImageViewType::TYPE_2D,
                    image_aspect_flags: ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: mip_levels,
                    base_array_layer: 0,
                    layer_count: 1,

                    layered: false,
                };

                let managed_image = self.resource_factories.managed_image_factory.allocate(
                    &resource_key,
                    image_description,
                    image_view_description,
                )?;

                let levels = transcode(
                    &reader,
                    self.texture_format.transcoder_block_format,
                )?;
                for (index, bc7_data) in levels.iter().enumerate() {
                    self.resource_loader.load_image(
                        managed_image.image,
                        Extent3D {
                            width: (reader.header().pixel_width >> index).max(1),
                            height: (reader.header().pixel_height >> index).max(1),
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
                        &bc7_data,
                    )?;
                }

                self.binding_layout.descriptor_set_manager.write(
                    GlobalDescriptorSetBindings::Texture,
                    *id,
                    &managed_image,
                    SamplerType::LinearClamp,
                );

                managed_image
            }
            ImageConfig::Inbuilt {
                label,

                image_description,
                image_view_description,

                binding,
                sampler_type,

                data,
            } => {
                let extent = image_description.extent;
                let managed_image = self.resource_factories.managed_image_factory.allocate(
                    &label,
                    image_description,
                    image_view_description,
                )?;

                if let Some(data) = data {
                    self.resource_loader.load_image(
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

                self.binding_layout.descriptor_set_manager.write(binding, *id, &managed_image, sampler_type);

                managed_image
            }
        };

        info!("Image resource {} is now available", id);

        Ok(managed_image)
    }

    fn erase(&self, _id: &ResourceId) -> Result<()> {
        // if let Some(default_image) = &self.default_image.get() {
        //     self.descriptor_set_manager.copy(
        //         GlobalDescriptorSetBindings::Texture,
        //         default_image.id,
        //         *id,
        //     );
        // } else {
        //     error!("ImageBackend default is None")
        // }

        Ok(())
    }

    fn statistics(&self) -> Self::Statistics {
        ()
    }
    
    fn destroy_resource(&self, resource: Self::Output) -> Result<()> {
        self.resource_factories.managed_image_factory.destroy_image(resource)?;

        Ok(())
    }

    fn destroy(self) -> Result<()> {
        Ok(())
    }
}
