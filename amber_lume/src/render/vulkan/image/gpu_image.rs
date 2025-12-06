use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::image::utils::find_memory_type_index;
use anyhow::Result;
use ash::vk::{
    DeviceMemory, Extent2D, Extent3D, ImageAspectFlags, ImageCreateInfo, ImageLayout,
    ImageSubresourceRange, ImageTiling, ImageType, ImageUsageFlags, ImageView, ImageViewCreateInfo,
    ImageViewType, MemoryAllocateInfo, MemoryPropertyFlags, PhysicalDevice, SampleCountFlags,
    Sampler, SharingMode,
};
use ash::{Device, Instance, vk};
use tracing::{info, instrument};
use vk::{Format, Image};

pub struct GpuImage {
    pub image: Image,
    pub memory: DeviceMemory,
    pub image_view: ImageView,

    pub sampler: Option<Sampler>,

    pub format: Format,
    pub extent: Extent2D,

    pub mip_levels: u32,
    pub array_layers: u32,

    pub samples: SampleCountFlags,
    pub aspect: ImageAspectFlags,
    pub usage: ImageUsageFlags,
}

impl GpuImage {
    #[instrument(skip_all)]
    pub fn create(
        instance: &Instance,
        device: &Device,
        physical_device: PhysicalDevice,

        extent: Extent2D,
        format: Format,

        mip_levels: u32,
        array_layers: u32,

        samples: SampleCountFlags,
        aspect: ImageAspectFlags,
        usage: ImageUsageFlags,
    ) -> Result<Self> {
        let image_create_info = ImageCreateInfo::default()
            .image_type(ImageType::TYPE_2D)
            .format(format)
            .extent(Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            })
            .mip_levels(mip_levels)
            .array_layers(array_layers)
            .samples(samples)
            .tiling(ImageTiling::OPTIMAL)
            .usage(usage)
            .sharing_mode(SharingMode::EXCLUSIVE)
            .initial_layout(ImageLayout::UNDEFINED);
        let image = unsafe { device.create_image(&image_create_info, None)? };

        let requirements = unsafe { device.get_image_memory_requirements(image) };
        let memory_property_flags = MemoryPropertyFlags::DEVICE_LOCAL;
        let memory_type_index = find_memory_type_index(
            &instance,
            physical_device,
            requirements.memory_type_bits,
            memory_property_flags,
        )?;
        let memory_allocate_info = MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type_index);
        let memory = unsafe { device.allocate_memory(&memory_allocate_info, None)? };
        unsafe { device.bind_image_memory(image, memory, 0)? };

        let image_subresource_range = ImageSubresourceRange::default()
            .aspect_mask(aspect)
            .base_mip_level(0)
            .level_count(mip_levels)
            .base_array_layer(0)
            .layer_count(array_layers);
        let image_view_create_info = ImageViewCreateInfo::default()
            .image(image)
            .view_type(ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(image_subresource_range);
        let image_view = unsafe { device.create_image_view(&image_view_create_info, None)? };

        let gpu_image = Self {
            image,
            memory,
            image_view,

            format,
            extent,

            sampler: None,

            mip_levels,
            array_layers,

            samples,
            aspect,
            usage,
        };

        Ok(gpu_image)
    }

    pub fn destroy(&self, device_context: &DeviceContext) -> Result<()> {
        if let Some(sampler) = self.sampler {
            unsafe { device_context.device.destroy_sampler(sampler, None) };
        }
        unsafe {
            device_context
                .device
                .destroy_image_view(self.image_view, None)
        };
        unsafe { device_context.device.destroy_image(self.image, None) };
        unsafe { device_context.device.free_memory(self.memory, None) };

        info!("GpuImage destroyed");

        Ok(())
    }
}
