use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::image::depth_gpu_image::DepthImage;
use crate::render::vulkan::image::gpu_image::GpuImage;
use crate::render::vulkan::swapchain::swapchain_context::SwapchainContext;
use crate::render::vulkan::vulkan_context::VulkanContext;
use anyhow::{Result, bail};
use ash::vk::{Extent2D, PhysicalDevice};
use ash::{Device, Instance, vk};
use tracing::info;
use vk::{Format, SampleCountFlags};

pub struct RenderTargets {
    pub depth_gpu_images: Vec<GpuImage>,
}

impl RenderTargets {
    pub fn create(
        vulkan_context: &VulkanContext,
        device_context: &DeviceContext,
        swapchain_context: &SwapchainContext,
    ) -> Result<Self> {
        let count = swapchain_context.image_views.len();

        let (_, depth_gpu_images) = create_depth_images(
            &vulkan_context.instance,
            &device_context.device,
            device_context.physical_device_info.handle,
            swapchain_context.extent,
            count,
            SampleCountFlags::TYPE_1,
        )?;

        info!("RenderTargets created");

        Ok(Self { depth_gpu_images })
    }

    pub fn get_depth_image(&self, image_index: usize) -> Result<&GpuImage> {
        let depth_gpu_image = self.depth_gpu_images.get(image_index);

        if let Some(depth_gpu_image) = depth_gpu_image {
            Ok(depth_gpu_image)
        } else {
            bail!("GpuImage index out of bounds");
        }
    }

    pub fn destroy(&self, device_context: &DeviceContext) -> Result<()> {
        for depth_image in &self.depth_gpu_images {
            depth_image.destroy(&device_context)?;
        }

        info!("RenderTargets destroyed");

        Ok(())
    }
}

fn create_depth_images(
    instance: &Instance,
    device: &Device,
    physical_device: PhysicalDevice,
    extent: Extent2D,
    count: usize,
    samples: SampleCountFlags,
) -> Result<(Format, Vec<GpuImage>)> {
    let mut vec = Vec::with_capacity(count);

    let format = DepthImage::find_depth_format(&instance, physical_device)?;

    for _ in 0..count {
        let image =
            DepthImage::create(&instance, &device, physical_device, format, extent, samples)?;

        vec.push(image)
    }

    Ok((format, vec))
}
