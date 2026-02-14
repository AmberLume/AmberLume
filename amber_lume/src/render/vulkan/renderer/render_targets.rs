use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::image::vulkan_image::VulkanImage;
use crate::render::vulkan::render_pass::depth::depth_format::find_depth_format;
use crate::render::vulkan::swapchain::swapchain_context::SwapchainContext;
use crate::render::vulkan::vulkan_context::VulkanContext;
use anyhow::Result;
use ash::vk;
use ash::vk::{Extent2D, ImageAspectFlags, ImageUsageFlags};
use tracing::info;
use vk::{Format, SampleCountFlags};

pub struct RenderTargets {
    pub depth_vulkan_image: VulkanImage,
}

impl RenderTargets {
    pub fn create(
        vulkan_context: &VulkanContext,
        device_context: &DeviceContext,
        swapchain_context: &SwapchainContext,
    ) -> Result<Self> {
        let format = find_depth_format(&vulkan_context, &device_context)?;

        let depth_vulkan_image = create_depth_image(
            device_context,
            swapchain_context.extent,
            format,
            SampleCountFlags::TYPE_1,
        )?;

        info!("RenderTargets created");

        Ok(Self { depth_vulkan_image })
    }

    pub fn destroy(self, device_context: &DeviceContext) -> Result<()> {
        self.depth_vulkan_image.destroy(device_context)?;

        info!("RenderTargets destroyed");

        Ok(())
    }
}

fn create_depth_image(
    device_context: &DeviceContext,
    extent: Extent2D,
    format: Format,
    samples: SampleCountFlags,
) -> Result<VulkanImage> {
    let depth_aspect = get_depth_aspect_mask(format);

    let vulkan_image = VulkanImage::create(
        device_context,
        "depth_image",
        extent,
        format,
        1,
        1,
        samples,
        depth_aspect,
        ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
    )?;

    Ok(vulkan_image)
}

fn get_depth_aspect_mask(format: Format) -> ImageAspectFlags {
    match format {
        Format::D16_UNORM | Format::D32_SFLOAT | Format::X8_D24_UNORM_PACK32 => {
            ImageAspectFlags::DEPTH
        }
        Format::D16_UNORM_S8_UINT | Format::D24_UNORM_S8_UINT | Format::D32_SFLOAT_S8_UINT => {
            ImageAspectFlags::DEPTH | ImageAspectFlags::STENCIL
        }
        Format::S8_UINT => ImageAspectFlags::STENCIL,
        _ => ImageAspectFlags::DEPTH,
    }
}
