use std::path::PathBuf;
use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::resource_context::ResourceContext;
use crate::render::vulkan::buffer::transfer_context::TransferContext;
use crate::resources::common::resource_backend::{ResourceBackend, ResourceKey};
use crate::resources::common::resource_provider::{ResourceId, ResourceProvider};
use crate::resources::index::resource_index::ResourceIndex;
use anyhow::{bail, Result};
use ash::vk::{BufferImageCopy, DescriptorImageInfo, DescriptorSet, DescriptorType, DeviceSize, Extent3D, Format, ImageAspectFlags, ImageLayout, ImageSubresourceLayers, ImageTiling, ImageType, ImageUsageFlags, ImageViewType, Offset3D, SampleCountFlags, SharingMode, WriteDescriptorSet};
use ktx2::{Reader, SupercompressionScheme};
use std::sync::{Arc, Mutex};
use ash::Device;
use basis_universal::{DecodeFlags, LowLevelUastcTranscoder, SliceParametersUastc, TranscoderBlockFormat};
use tracing::info;
use zstd::bulk::decompress;
use crate::render::vulkan::factories::image::managed_image::{ImageDescription, ImageViewDescription, ManagedImage};
use crate::resources::descriptor_set::descriptor_set_backend::DescriptorSetBackend;
use crate::resources::descriptor_set::descriptor_set_config::DescriptorSetConfig;
use crate::resources::descriptor_set_layout::descriptor_set_layout_config::DescriptorSetLayoutConfig;
use crate::resources::image::image_config::ImageConfig;
use crate::resources::resource_factories::ResourceFactories;
use crate::resources::sampler::sampler_backend::SamplerBackend;

pub struct ImageBackend {
    device: Device,
    resource_factories: Arc<ResourceFactories>,

    sampler_provider: Arc<ResourceProvider<SamplerBackend>>,
    descriptor_set_provider: Arc<ResourceProvider<DescriptorSetBackend>>,

    large_transfer_context: Arc<Mutex<Option<TransferContext>>>,

    buffer_manager: Arc<BufferManager>,

    resource_index: Arc<ResourceIndex>,
}

impl ImageBackend {
    pub fn new(
        device: Device,
        resource_factories: Arc<ResourceFactories>,
        resource_context: &ResourceContext,
        sampler_provider: Arc<ResourceProvider<SamplerBackend>>,
        descriptor_set_provider: Arc<ResourceProvider<DescriptorSetBackend>>,
        resource_index: Arc<ResourceIndex>,
    ) -> Self {
        Self {
            device,
            resource_factories,
            
            sampler_provider,
            descriptor_set_provider,

            large_transfer_context: resource_context.large_transfer_context.clone(),

            buffer_manager: resource_context.buffer_manager.clone(),

            resource_index,
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
        let mut transfer_context_guard = self.large_transfer_context.lock().unwrap();
        let Some(transfer_context) = transfer_context_guard.as_mut() else {
            bail!("transfer context is None");
        };
        
        transfer_context.begin()?;

        let transcoder = LowLevelUastcTranscoder::new();
        let mut actual_copies = Vec::new();

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

            transfer_context.align(16)?;
            let buffer_offset = transfer_context.stage(&bc7_data)?;

            actual_copies.push(BufferImageCopy {
                buffer_offset: buffer_offset as DeviceSize,
                buffer_row_length: 0,
                buffer_image_height: 0,
                image_subresource: ImageSubresourceLayers {
                    aspect_mask: ImageAspectFlags::COLOR,
                    mip_level: index as u32,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                image_offset: Offset3D::default(),
                image_extent: Extent3D {
                    width: (reader.header().pixel_width >> index).max(1),
                    height: (reader.header().pixel_height >> index).max(1),
                    depth: 1
                },
            });
        }

        transfer_context.flush_to_image(managed_image.image, managed_image.image_description.mip_levels, &actual_copies)?;
        transfer_context.submit()
    }
}

pub struct ImageDependencies {
    descriptor_set: DescriptorSet,
}

impl ResourceBackend for ImageBackend {
    type Config = ImageConfig;
    type Dependencies = ImageDependencies;
    type Output = ManagedImage;

    fn key_from(config: &Self::Config) -> ResourceKey {
        config.hash()
    }

    fn collect_dependencies(&self, _config: &Self::Config) -> Self::Dependencies {
        let descriptor_set = self.descriptor_set_provider.get_now(&DescriptorSetConfig {
            descriptor_set_layout_config: DescriptorSetLayoutConfig::default(),
        });

        Self::Dependencies {
            descriptor_set: *descriptor_set,
        }
    }

    fn create(
        &self,
        id: &ResourceId,
        config: Self::Config,
        dependencies: Self::Dependencies,
    ) -> Result<Self::Output> {
        let sampler = self.sampler_provider.get_now(&config.sampler_config);

        let image_name = PathBuf::from("textures").join(&config.name).with_extension("ktx2").to_string_lossy().to_string();
        let image_bytes = self.resource_index.get_resource(&image_name)?;

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
        };

        let managed_image = self.resource_factories.managed_image_factory.allocate(
            &config.name,
            image_description,
            image_view_description,
        )?;

        self.push_to_buffer_from_reader(&reader, &managed_image)?;

        self.buffer_manager.image_availability_buffer.stage(*id as usize, &[1u32])?;
        info!("Image resource {} is now available", id);

        let image_info = DescriptorImageInfo::default()
            .image_layout(ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(managed_image.image_view)
            .sampler(*sampler);

        let image_info = [image_info];
        let write = WriteDescriptorSet::default()
            .dst_set(dependencies.descriptor_set)
            .dst_binding(0)
            .dst_array_element(*id as u32)
            .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&image_info);

        unsafe { self.device.update_descriptor_sets(&[write], &[]) };

        Ok(managed_image)
    }

    fn destroy_resource(&self, resource: Self::Output) -> Result<()> {
        self.resource_factories.managed_image_factory.destroy(resource)?;

        Ok(())
    }

    fn destroy(&mut self) -> Result<()> {
        Ok(())
    }
}
