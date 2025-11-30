use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::image::depth_gpu_image::DepthImage;
use crate::render::vulkan::image::gpu_image::GpuImage;
use crate::render::vulkan::swapchain::Swapchain;
use crate::render::vulkan::vk_context::VkContext;
use anyhow::Result;
use ash::vk::{Extent2D, PhysicalDevice};
use ash::{Device, Instance, vk};
use vk::{Format, SampleCountFlags};

pub struct RenderTargets {
    pub depth_gpu_images: Vec<GpuImage>,
}

impl RenderTargets {
    pub fn create(
        vk_context: &VkContext,
        device_context: &DeviceContext,
        swapchain: &Swapchain,
    ) -> Result<Self> {
        let count = swapchain.image_views.len();

        let (_, depth_gpu_images) = create_depth_images(
            &vk_context.instance,
            &device_context.device,
            device_context.physical_device_info.handle,
            swapchain.extent,
            count,
            SampleCountFlags::TYPE_1,
        )?;

        Ok(Self { depth_gpu_images })
    }

    pub fn destroy(&self, device: &Device) {
        for depth_image in &self.depth_gpu_images {
            depth_image.destroy(device);
        }
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
