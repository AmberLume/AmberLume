use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;
use ash::khr::swapchain::Device;
use ash::vk::{
    Extent2D, Extent3D, Format, Image, ImageAspectFlags, ImageCreateInfo, ImageLayout,
    ImageSubresourceRange, ImageTiling, ImageType, ImageUsageFlags, ImageView, ImageViewCreateInfo,
    ImageViewType, SampleCountFlags, SharingMode, SurfaceFormatKHR, SwapchainKHR,
};
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme};
use tracing::info;

#[derive(Debug)]
pub struct VulkanImage {
    pub format: Format,
    pub extent: Extent2D,

    pub image: Image,
    pub image_view: ImageView,

    pub image_subresource_range: ImageSubresourceRange,

    allocation: Option<Allocation>,
}

impl VulkanImage {
    pub fn from_swapchain(
        device_context: &DeviceContext,
        swapchain_loader: &Device,
        swapchain: SwapchainKHR,
        surface_format: SurfaceFormatKHR,
        extent: Extent2D,
    ) -> Result<Vec<Self>> {
        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };

        let mut vulkan_images: Vec<VulkanImage> = Vec::with_capacity(images.len());

        for image in images {
            let image_subresource_range =
                Self::create_image_subresource_range(ImageAspectFlags::COLOR, 0, 1, 0, 1);
            let image_view = Self::create_image_view(
                &device_context,
                image,
                ImageViewType::TYPE_2D,
                surface_format.format,
                image_subresource_range,
            )?;

            let vulkan_image = VulkanImage {
                extent,
                format: surface_format.format,

                image,
                image_view,

                image_subresource_range,

                allocation: None,
            };

            info!("VulkanImage created: {:?}", vulkan_image);

            vulkan_images.push(vulkan_image);
        }

        Ok(vulkan_images)
    }

    pub fn create(
        device_context: &DeviceContext,

        name: &str,

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
        let image = unsafe {
            device_context
                .device
                .create_image(&image_create_info, None)?
        };

        let requirements = unsafe { device_context.device.get_image_memory_requirements(image) };
        
        let allocation = {
            let mut allocator = device_context.allocator.lock().unwrap();
            
            allocator.allocate(&AllocationCreateDesc {
                name,
                requirements,
                location: MemoryLocation::GpuOnly,
                linear: false,
                allocation_scheme: AllocationScheme::GpuAllocatorManaged,
            })?
        };

        unsafe {
            device_context.device.bind_image_memory(
                image,
                allocation.memory(),
                allocation.offset(),
            )?
        };

        let image_subresource_range =
            Self::create_image_subresource_range(aspect, 0, mip_levels, 0, array_layers);
        let image_view = Self::create_image_view(
            &device_context,
            image,
            ImageViewType::TYPE_2D,
            format,
            image_subresource_range,
        )?;

        let vulkan_image = Self {
            format,
            extent,

            image,
            image_view,

            image_subresource_range,

            allocation: Some(allocation),
        };

        Ok(vulkan_image)
    }

    fn create_image_subresource_range(
        aspect_mask: ImageAspectFlags,
        base_mip_level: u32,
        level_count: u32,
        base_array_layer: u32,
        layer_count: u32,
    ) -> ImageSubresourceRange {
        ImageSubresourceRange::default()
            .aspect_mask(aspect_mask)
            .base_mip_level(base_mip_level)
            .level_count(level_count)
            .base_array_layer(base_array_layer)
            .layer_count(layer_count)
    }

    fn create_image_view(
        device_context: &DeviceContext,
        image: Image,
        view_type: ImageViewType,
        format: Format,
        image_subresource_range: ImageSubresourceRange,
    ) -> Result<ImageView> {
        let image_create_info = ImageViewCreateInfo::default()
            .image(image)
            .view_type(view_type)
            .format(format)
            .subresource_range(image_subresource_range);

        let image_view = unsafe {
            device_context
                .device
                .create_image_view(&image_create_info, None)?
        };

        Ok(image_view)
    }

    pub fn destroy(self, device_context: &DeviceContext) -> Result<()> {
        unsafe {
            device_context
                .device
                .destroy_image_view(self.image_view, None)
        };

        if let Some(allocation) = self.allocation {
            unsafe { device_context.device.destroy_image(self.image, None) };
            
            let mut allocator = device_context.allocator.lock().unwrap();
            allocator.free(allocation)?;
        }

        info!("VulkanImage destroyed");

        Ok(())
    }
}
