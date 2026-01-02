use std::mem::ManuallyDrop;
use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::resource_context::ResourceContext;
use crate::render::vulkan::buffer::transfer_context::TransferContext;
use crate::resources::common::resource_backend::{ResourceBackend, ResourceKey};
use crate::resources::common::resource_provider::{ResourceId, ResourceProvider};
use crate::resources::index::resource_index::ResourceIndex;
use anyhow::{bail, Result};
use ash::vk::{BufferImageCopy, DescriptorImageInfo, DescriptorSet, DescriptorType, DeviceSize, Extent3D, Format, Image, ImageAspectFlags, ImageCreateInfo, ImageLayout, ImageSubresourceLayers, ImageSubresourceRange, ImageTiling, ImageType, ImageUsageFlags, ImageView, ImageViewCreateInfo, ImageViewType, Offset3D, SampleCountFlags, Sampler, SharingMode, WriteDescriptorSet};
use ktx2::{Reader, SupercompressionScheme};
use std::sync::{Arc, Mutex};
use ash::Device;
use basis_universal::{DecodeFlags, LowLevelUastcTranscoder, SliceParametersUastc, TranscoderBlockFormat};
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator};
use tracing::info;
use zstd::bulk::decompress;
use crate::render::vulkan::debug_utils::DebugUtils;
use crate::render::vulkan::device_context::DeviceContext;
use crate::resources::descriptor_set::descriptor_set_backend::DescriptorSetBackend;
use crate::resources::descriptor_set::descriptor_set_config::DescriptorSetConfig;
use crate::resources::descriptor_set_layout::descriptor_set_layout_config::DescriptorSetLayoutConfig;
use crate::resources::image::image_config::ImageConfig;
use crate::resources::sampler::sampler_backend::SamplerBackend;
use crate::resources::sampler::sampler_config::SamplerConfig;

pub struct ImageBackend {
    device: Device,
    debug_utils: Arc<DebugUtils>,
    allocator: Arc<Mutex<ManuallyDrop<Allocator>>>,

    sampler_provider: Arc<ResourceProvider<SamplerBackend>>,
    descriptor_set_provider: Arc<ResourceProvider<DescriptorSetBackend>>,

    large_transfer_context: Arc<Mutex<TransferContext>>,

    buffer_manager: Arc<BufferManager>,

    resource_index: Arc<ResourceIndex>,
}

impl ImageBackend {
    pub fn new(
        device_context: &DeviceContext,
        resource_context: &ResourceContext,
        sampler_provider: Arc<ResourceProvider<SamplerBackend>>,
        descriptor_set_provider: Arc<ResourceProvider<DescriptorSetBackend>>,
        resource_index: Arc<ResourceIndex>,
    ) -> Self {
        Self {
            device: device_context.device.clone(),
            debug_utils: device_context.debug_utils.clone(),
            allocator: device_context.allocator.clone(),

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

    fn create_image(
        &self,
        label: &str,
        width: u32,
        height: u32,
        format: Format,
        mip_levels: u32,
    ) -> Result<Image> {
        let image_info = ImageCreateInfo::default()
            .image_type(ImageType::TYPE_2D)
            .format(format)
            .extent(Extent3D { width, height, depth: 1 })
            .mip_levels(mip_levels)
            .array_layers(1)
            .samples(SampleCountFlags::TYPE_1)
            .tiling(ImageTiling::OPTIMAL)
            .usage(ImageUsageFlags::TRANSFER_DST | ImageUsageFlags::SAMPLED)
            .sharing_mode(SharingMode::EXCLUSIVE);

        let image = unsafe { self.device.create_image(&image_info, None)? };

        self.debug_utils.label(image, &format!("image: {}", label));

        Ok(image)
    }

    fn create_allocation(&self, name: &str, image: Image) -> Result<Allocation> {
        let mem_reqs = unsafe { self.device.get_image_memory_requirements(image) };
        let allocation = self.allocator.lock().unwrap().allocate(&AllocationCreateDesc {
            name,
            requirements: mem_reqs,
            location: MemoryLocation::GpuOnly,
            linear: false,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        })?;

        unsafe { self.device.bind_image_memory(image, allocation.memory(), allocation.offset())? };

        Ok(allocation)
    }

    fn push_to_buffer(
        &self,
        reader: &Reader<&[u8]>,
        image: Image,
        mip_levels: u32,
    ) -> Result<()> {
        let mut transfer_context = self.large_transfer_context.lock().unwrap();
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

            transfer_context.align_staging(16);
            let buffer_offset = transfer_context.get_current_offset();
            transfer_context.stage(&bc7_data)?;

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

        transfer_context.copy_stage_to_image(image, mip_levels, actual_copies)?;
        transfer_context.submit()
    }

    fn create_image_view(
        &self,
        label: &str,
        image: Image,
        mip_levels: u32,
        format: Format,
    ) -> Result<ImageView> {
        let image_subresource_range = ImageSubresourceRange::default()
            .aspect_mask(ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(mip_levels)
            .base_array_layer(0)
            .layer_count(1);

        let view_info = ImageViewCreateInfo::default()
            .image(image)
            .view_type(ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(image_subresource_range);

        let view = unsafe { self.device.create_image_view(&view_info, None)? };

        self.debug_utils.label(view, &format!("image_view: {}", label));

        Ok(view)
    }
}

pub struct ImageDependencies {
    sampler: Sampler,
    descriptor_set: DescriptorSet,
}

#[derive(Clone)]
pub struct ImageResource {
    pub image: Image,
    pub image_view: ImageView,
    pub allocation: Arc<Allocation>,
}

impl ResourceBackend for ImageBackend {
    type Config = ImageConfig;
    type Dependencies = ImageDependencies;
    type Output = ImageResource;

    fn key_from(config: &Self::Config) -> ResourceKey {
        config.hash()
    }

    fn collect_dependencies(&self, _config: &Self::Config) -> Self::Dependencies {
        let sampler = self.sampler_provider.get_now(&SamplerConfig::default());
        let descriptor_set = self.descriptor_set_provider.get_now(&DescriptorSetConfig {
            descriptor_set_layout_config: DescriptorSetLayoutConfig::default(),
        });

        Self::Dependencies {
            sampler: *sampler,
            descriptor_set: *descriptor_set,
        }
    }

    fn create(
        &self,
        id: &ResourceId,
        config: Self::Config,
        dependencies: Self::Dependencies,
    ) -> Result<Self::Output> {
        let image_bytes = self.resource_index.get_resource(&config.name).unwrap();

        let reader = Reader::new(image_bytes)?;
        let header = reader.header();

        let width = header.pixel_width;
        let height = header.pixel_height;
        let mip_levels = header.level_count;
        if reader.header().supercompression_scheme != Some(SupercompressionScheme::Zstandard) {
            bail!("Unsupported supercompression scheme: {:?}", reader.header().supercompression_scheme);
        }
        let format = Format::BC7_SRGB_BLOCK;

        let image = self.create_image(&config.name, width, height, format, mip_levels)?;
        let allocation = self.create_allocation(&config.name, image)?;

        self.push_to_buffer(&reader, image, mip_levels)?;

        let image_view = self.create_image_view(&config.name, image, mip_levels, format)?;

        self.buffer_manager.image_availability_buffer.set_availability(*id, 1u32)?;
        info!("Image resource {} is now available", id);

        let image_info = DescriptorImageInfo::default()
            .image_layout(ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(image_view)
            .sampler(dependencies.sampler);

        let image_info = [image_info];
        let write = WriteDescriptorSet::default()
            .dst_set(dependencies.descriptor_set)
            .dst_binding(0)
            .dst_array_element(*id as u32)
            .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&image_info);

        unsafe { self.device.update_descriptor_sets(&[write], &[]) };

        Ok(ImageResource {
            image,
            image_view,
            allocation: Arc::new(allocation),
        })
    }

    fn destroy_resource(&self, resource: Self::Output) -> Result<()> {
        unsafe { self.device.destroy_image_view(resource.image_view, None) }
        unsafe { self.device.destroy_image(resource.image, None) }

        let mut allocator = self.allocator.lock().unwrap();
        allocator.free(Arc::try_unwrap(resource.allocation).unwrap())?;

        Ok(())
    }

    fn destroy(&mut self) -> Result<()> {
        Ok(())
    }
}
