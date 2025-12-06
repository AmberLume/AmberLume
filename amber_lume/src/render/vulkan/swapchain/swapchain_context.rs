use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::surface::surface_provider::SurfaceProvider;
use crate::render::vulkan::surface::vulkan_surface::VulkanSurface;
use crate::render::vulkan::swapchain::extent::create_extent;
use crate::render::vulkan::swapchain::image_views::create_image_views;
use crate::render::vulkan::swapchain::present_mode::get_present_mode;
use crate::render::vulkan::swapchain::surface_capabilities::create_surface_capabilities;
use crate::render::vulkan::swapchain::surface_format::get_surface_format;
use crate::render::vulkan::swapchain::swapchain::create_swapchain;
use crate::render::vulkan::vulkan_context::VulkanContext;
use anyhow::Result;
use ash::khr::swapchain::Device;
use ash::vk::{Extent2D, Format, Image, ImageView, SwapchainKHR};
use std::sync::Arc;
use tracing::info;

pub struct SwapchainContext {
    pub handle: SwapchainKHR,

    pub loader: Device,

    pub format: Format,
    pub extent: Extent2D,

    pub images: Vec<Image>,
    pub image_views: Vec<ImageView>,
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

        let (images, image_views) =
            create_image_views(&loader, swapchain, surface_format, &device_context.device)?;

        info!("SwapchainContext created");

        Ok(Self {
            handle: swapchain,

            loader,

            format: surface_format.format,
            extent,

            images,
            image_views,
        })
    }

    pub fn teardown_and_setup(
        &mut self,
        vulkan_context: &VulkanContext,
        vulkan_surface: &VulkanSurface,
        device_context: &DeviceContext,
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

        let (images, image_views) = create_image_views(
            &self.loader,
            swapchain,
            surface_format,
            &device_context.device,
        )?;

        info!("SwapchainContext recreated. Destroying old one...");

        self.destroy(&device_context)?;

        self.images = images;
        self.image_views = image_views;
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

    pub fn get_image(&self, index: usize) -> Result<Image> {
        let image = self.images[index];

        Ok(image)
    }

    pub fn get_image_view(&self, index: usize) -> Result<ImageView> {
        let image_view = self.image_views[index];

        Ok(image_view)
    }

    pub fn destroy(&self, device_context: &DeviceContext) -> Result<()> {
        for &image_view in &self.image_views {
            unsafe { device_context.device.destroy_image_view(image_view, None) };
        }
        unsafe { self.loader.destroy_swapchain(self.handle, None) };

        info!("SwapchainContext destroyed");

        Ok(())
    }
}
