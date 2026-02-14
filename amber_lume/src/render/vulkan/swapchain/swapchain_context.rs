use crate::platform_providers::surface_provider::SurfaceProvider;
use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::image::vulkan_image::VulkanImage;
use crate::render::vulkan::surface::vulkan_surface::VulkanSurface;
use crate::render::vulkan::swapchain::extent::create_extent;
use crate::render::vulkan::swapchain::present_mode::get_present_mode;
use crate::render::vulkan::swapchain::surface_capabilities::create_surface_capabilities;
use crate::render::vulkan::swapchain::surface_format::get_surface_format;
use crate::render::vulkan::swapchain::swapchain::create_swapchain;
use crate::render::vulkan::vulkan_context::VulkanContext;
use anyhow::{Result, bail};
use ash::khr::swapchain::Device;
use ash::vk::{Extent2D, Format, SwapchainKHR};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::info;

pub struct SwapchainContext {
    pub handle: SwapchainKHR,

    pub loader: Device,

    pub format: Format,
    pub extent: Extent2D,

    pub vulkan_images: Vec<VulkanImage>,

    pub is_out_of_date: AtomicBool,
}

impl SwapchainContext {
    pub fn create(
        old: Option<&Self>,
        vulkan_context: &VulkanContext,
        vulkan_surface: &VulkanSurface,
        device_context: &DeviceContext,
        surface_provider: Arc<dyn SurfaceProvider>,
    ) -> Result<Self> {
        let loader = if let Some(old) = &old {
            old.loader.clone()
        } else {
            Self::create_loader(&vulkan_context, &device_context)?
        };

        let surface_capabilities = create_surface_capabilities(
            &vulkan_context,
            &vulkan_surface,
            &device_context.physical_device_info,
        )?;
        let surface_format = get_surface_format(
            &vulkan_context,
            &vulkan_surface,
            &device_context.physical_device_info,
        )?;
        let present_mode = get_present_mode(
            &vulkan_context,
            &vulkan_surface,
            &device_context.physical_device_info,
        )?;
        let extent = create_extent(&surface_capabilities, surface_provider)?;

        let swapchain = create_swapchain(
            &vulkan_surface,
            &loader,
            &surface_capabilities,
            &device_context.queues,
            &surface_format,
            extent,
            present_mode,
            old.map(|old| old.handle),
        )?;

        let vulkan_images = VulkanImage::from_swapchain(
            &device_context,
            &loader,
            swapchain,
            surface_format,
            extent,
        )?;

        info!("SwapchainContext created");

        Ok(Self {
            handle: swapchain,

            loader,

            format: surface_format.format,
            extent,

            vulkan_images,

            is_out_of_date: AtomicBool::new(false),
        })
    }

    fn create_loader(
        vulkan_context: &VulkanContext,
        device_context: &DeviceContext,
    ) -> Result<Device> {
        let loader = Device::new(&vulkan_context.instance, &device_context.device);

        Ok(loader)
    }

    pub fn get_image(&self, index: usize) -> Result<&VulkanImage> {
        if let Some(vulkan_image) = self.vulkan_images.get(index) {
            Ok(vulkan_image)
        } else {
            bail!("Swapchain VulkanImage index out of bounds");
        }
    }

    pub fn set_is_out_of_date(&self, state: bool) {
        self.is_out_of_date.store(state, Ordering::Relaxed);
    }

    pub fn destroy(self, device_context: &DeviceContext) -> Result<()> {
        for vulkan_image in self.vulkan_images {
            vulkan_image.destroy(device_context)?;
        }
        unsafe { self.loader.destroy_swapchain(self.handle, None) };

        info!("SwapchainContext destroyed");

        Ok(())
    }
}
