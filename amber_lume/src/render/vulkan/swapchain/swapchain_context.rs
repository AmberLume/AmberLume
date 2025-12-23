use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::image::vulkan_image::VulkanImage;
use crate::render::vulkan::surface::surface_provider::SurfaceProvider;
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
        vulkan_context: &VulkanContext,
        vulkan_surface: &VulkanSurface,
        device_context: &DeviceContext,
        surface_provider: Arc<dyn SurfaceProvider>,
    ) -> Result<Self> {
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
        let surface_size = surface_provider.size();
        let extent = create_extent(&surface_capabilities, &surface_size)?;

        let loader = Self::create_loader(&vulkan_context, &device_context)?;

        let swapchain = create_swapchain(
            &vulkan_surface,
            &loader,
            &surface_capabilities,
            &device_context.queue_families,
            &surface_format,
            extent,
            present_mode,
            None,
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

    pub fn teardown_and_setup(
        &mut self,
        vulkan_context: &VulkanContext,
        vulkan_surface: &VulkanSurface,
        device_context: &mut DeviceContext,
        surface_provider: Arc<dyn SurfaceProvider>,
    ) -> Result<()> {
        unsafe { device_context.device.device_wait_idle() }?;

        let surface_capabilities = create_surface_capabilities(
            vulkan_context,
            vulkan_surface,
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
        let surface_size = surface_provider.size();
        let extent = create_extent(&surface_capabilities, &surface_size)?;

        let swapchain = create_swapchain(
            &vulkan_surface,
            &self.loader,
            &surface_capabilities,
            &device_context.queue_families,
            &surface_format,
            extent,
            present_mode,
            Some(self.handle),
        )?;

        let vulkan_images = VulkanImage::from_swapchain(
            &device_context,
            &self.loader,
            swapchain,
            surface_format,
            extent,
        )?;

        info!("SwapchainContext recreated. Destroying old one...");

        self.destroy(device_context)?;

        self.vulkan_images = vulkan_images;
        self.handle = swapchain;
        self.format = surface_format.format;
        self.extent = extent;

        Ok(())
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

    pub fn destroy(&mut self, device_context: &mut DeviceContext) -> Result<()> {
        for vulkan_image in self.vulkan_images.iter_mut() {
            vulkan_image.destroy(device_context)?;
        }
        unsafe { self.loader.destroy_swapchain(self.handle, None) };

        info!("SwapchainContext destroyed");

        Ok(())
    }
}
