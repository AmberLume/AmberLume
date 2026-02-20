use crate::platform_providers::surface_provider::SurfaceProvider;
use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::surface::vulkan_surface::VulkanSurface;
use crate::render::vulkan::swapchain::extent::create_extent;
use crate::render::vulkan::swapchain::present_mode::get_present_mode;
use crate::render::vulkan::swapchain::surface_capabilities::create_surface_capabilities;
use crate::render::vulkan::swapchain::surface_format::get_surface_format;
use crate::render::vulkan::swapchain::swapchain::create_swapchain;
use crate::render::vulkan::vulkan_context::VulkanContext;
use anyhow::{Result, bail};
use ash::vk::{Extent2D, Format, SwapchainKHR};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use ash::Device;
use tracing::info;
use crate::render::vulkan::factories::image::swapchain_image::SwapchainImage;
use crate::render::vulkan::types::SwapchainDevice;

pub struct SwapchainContext {
    pub handle: SwapchainKHR,

    pub loader: SwapchainDevice,

    pub format: Format,
    pub extent: Extent2D,

    pub swapchain_images: Vec<SwapchainImage>,

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

        let swapchain_images = SwapchainImage::create(
            &device_context.device,
            &loader,
            swapchain,
            surface_format.format,
            extent,
        )?;

        info!("SwapchainContext created");

        Ok(Self {
            handle: swapchain,

            loader,

            format: surface_format.format,
            extent,

            swapchain_images,

            is_out_of_date: AtomicBool::new(false),
        })
    }

    fn create_loader(
        vulkan_context: &VulkanContext,
        device_context: &DeviceContext,
    ) -> Result<SwapchainDevice> {
        let loader = SwapchainDevice::new(&vulkan_context.instance, &device_context.device);

        Ok(loader)
    }

    pub fn get_image(&self, index: u32) -> Result<&SwapchainImage> {
        if let Some(image) = self.swapchain_images.get(index as usize) {
            Ok(image)
        } else {
            bail!("SwapchainImage index out of bounds");
        }
    }

    pub fn set_is_out_of_date(&self, state: bool) {
        self.is_out_of_date.store(state, Ordering::Relaxed);
    }

    pub fn destroy(self, device: &Device) -> Result<()> {
        for image in self.swapchain_images {
            image.destroy(&device)?;
        }
        unsafe { self.loader.destroy_swapchain(self.handle, None) };

        info!("SwapchainContext destroyed");

        Ok(())
    }
}
