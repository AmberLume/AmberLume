use crate::render::vulkan::surface::surface_provider::SurfaceProvider;
use crate::render::vulkan::vulkan_context::VulkanContext;
use anyhow::Result;
use ash::vk;
use ash_window::create_surface;
use std::sync::Arc;
use tracing::info;
use vk::SurfaceKHR;

pub struct VulkanSurface {
    pub surface: SurfaceKHR,
}

impl VulkanSurface {
    pub fn create(
        vulkan_context: &VulkanContext,
        surface_provider: Arc<dyn SurfaceProvider>,
    ) -> Result<Self> {
        let surface = Self::create_surface(&vulkan_context, surface_provider)?;

        info!("VulkanSurface created");

        Ok(Self { surface })
    }

    fn create_surface(
        vulkan_context: &VulkanContext,
        surface_provider: Arc<dyn SurfaceProvider>,
    ) -> Result<SurfaceKHR> {
        let (raw_display_handle, raw_window_handle) = surface_provider.handles();

        let surface = unsafe {
            create_surface(
                &vulkan_context.entry,
                &vulkan_context.instance,
                raw_display_handle,
                raw_window_handle,
                None,
            )
        }?;

        Ok(surface)
    }

    pub fn destroy(&self, vulkan_context: &VulkanContext) -> Result<()> {
        unsafe {
            vulkan_context
                .surface_loader
                .destroy_surface(self.surface, None);
        }

        info!("VulkanSurface destroyed");

        Ok(())
    }
}
