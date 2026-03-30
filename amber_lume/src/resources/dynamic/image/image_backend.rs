use crate::resources::alpaca_resource_reader::alpaca_resource_reader::AlpacaResourceReader;
use anyhow::{bail, Result};
use ash::vk::{Extent3D, Format, ImageAspectFlags, ImageSubresourceLayers, ImageTiling, ImageType, ImageUsageFlags, ImageViewType, SampleCountFlags, SharingMode};
use ktx2::{Reader, SupercompressionScheme};
use std::sync::Arc;
use basis_universal::{DecodeFlags, LowLevelUastcTranscoder, SliceParametersUastc, TranscoderBlockFormat};
use tracing::info;
use zstd::bulk::decompress;
use crate::render::factories::image::managed_image::{ImageDescription, ImageViewDescription, ManagedImage};
use crate::render::resources::resource_loader::ResourceLoader;
use crate::resources::descriptor_set_manager::{DescriptorSetManager, GlobalDescriptorSetBindings};
use crate::resources::dynamic::image::image_config::ImageConfig;
use crate::resources::dynamic::resource_backend::{ResourceBackend, ResourceKey};
use crate::resources::dynamic::resource_provider::ResourceId;
use crate::resources::persistent::persistent_resources::PersistentResources;
use crate::resources::resource_factories::ResourceFactories;
use crate::resources::sampler_registry::SamplerType;

pub struct ImageBackend {
    resource_factories: Arc<ResourceFactories>,
    alpaca_resource_reader: Arc<AlpacaResourceReader>,

    persistent_resources: Arc<PersistentResources>,
    descriptor_set_manager: Arc<DescriptorSetManager>,

    resource_loader: Arc<ResourceLoader>,
}

impl ImageBackend {
    pub fn new(
        resource_factories: Arc<ResourceFactories>,
        alpaca_resource_reader: Arc<AlpacaResourceReader>,
        persistent_resources: Arc<PersistentResources>,
        descriptor_set_manager: Arc<DescriptorSetManager>,
        resource_loader: Arc<ResourceLoader>,
    ) -> Self {
        Self {
            resource_factories,
            alpaca_resource_reader,

            persistent_resources,
            descriptor_set_manager,

            resource_loader,
        }
    }

    fn calculate_uastc_uncompressed_size(&self, reader: &Reader<&[u8]>, level_index: u32) -> usize {
        let header = reader.header();
        let width = (header.pixel_width >> level_index).max(1);
        let height = (header.pixel_height >> level_index).max(1);

        let blocks_x = (width + 3) / 4;
        let blocks_y = (height + 3) / 4;

        (blocks_x * blocks_y * 16) as usize
    }

    fn push_to_buffer_from_reader(
        &self,
        reader: &Reader<&[u8]>,
        managed_image: &ManagedImage,
    ) -> Result<()> {
        let transcoder = LowLevelUastcTranscoder::new();

        for (index, level) in reader.levels().enumerate() {
            let level_width = (reader.header().pixel_width >> index).max(1);
            let level_height = (reader.header().pixel_height >> index).max(1);

            let num_blocks_x = (level_width + 3) / 4;
            let num_blocks_y = (level_height + 3) / 4;

            let uncompressed_size = self.calculate_uastc_uncompressed_size(reader, index as u32);

            let uastc_data = decompress(level.data, uncompressed_size)?;

            let slice_params = SliceParametersUastc {
                num_blocks_x,
                num_blocks_y,
                has_alpha: true,
                original_width: level_width,
                original_height: level_height,
            };

            let bc7_data = transcoder
                .transcode_slice(
                    &uastc_data,
                    slice_params,
                    DecodeFlags::empty(),
                    TranscoderBlockFormat::BC7,
                )
                .map_err(|e| anyhow::anyhow!("Low-level transcode failed: {:?}", e))?;

            self.resource_loader.load_image(
                managed_image.image,
                Extent3D {
                    width: (reader.header().pixel_width >> index).max(1),
                    height: (reader.header().pixel_height >> index).max(1),
                    depth: 1
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

        Ok(())
    }
}

impl ResourceBackend for ImageBackend {
    type Config = ImageConfig;
    type Output = ManagedImage;

    fn key_from(config: &Self::Config) -> ResourceKey {
        config.hash()
    }

    fn create(
        &self,
        id: &ResourceId,
        config: Self::Config,
    ) -> Result<Self::Output> {
        let image_bytes = self.alpaca_resource_reader.get_resource(&config.resource_key)?;

        let reader = Reader::new(image_bytes)?;
        let header = reader.header();

        let width = header.pixel_width;
        let height = header.pixel_height;

        let mip_levels = header.level_count;

        if reader.header().supercompression_scheme != Some(SupercompressionScheme::Zstandard) {
            bail!("Unsupported supercompression scheme: {:?}", reader.header().supercompression_scheme);
        }
        let format = Format::BC7_SRGB_BLOCK;

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
            &config.resource_key,
            image_description,
            image_view_description,
        )?;

        self.push_to_buffer_from_reader(&reader, &managed_image)?;

        info!("Image resource {} is now available", id);

        Ok(managed_image)
    }

    fn set_default(&self, id: &ResourceId) -> Result<()> {
        self.descriptor_set_manager.write(
            GlobalDescriptorSetBindings::Texture,
            *id,
            &self.persistent_resources.images.white_pixel.managed_image,
            SamplerType::LinearClamp,
        );

        Ok(())
    }

    fn set_resource(&self, id: &ResourceId, output: &Self::Output) -> Result<()> {
        self.descriptor_set_manager.write(
            GlobalDescriptorSetBindings::Texture,
            *id,
            &output,
            SamplerType::LinearClamp,
        );
        
        Ok(())
    }

    fn destroy_resource(&self, resource: Self::Output) -> Result<()> {
        self.resource_factories.managed_image_factory.destroy_image(resource)?;

        Ok(())
    }
}
