use crate::render::vulkan::surface_provider::SurfaceProvider;
use crate::render::vulkan::vk_context::VkContext;
use anyhow::Context;
use anyhow::Result;
use ash::vk;
use ash_window::create_surface;
use std::sync::Arc;
use tracing::{info, instrument};
use vk::SurfaceKHR;

pub struct VkSurface {
    vk_context: Arc<VkContext>,

    pub surface: SurfaceKHR,
}

impl VkSurface {
    #[instrument(level = "trace", skip(vk_context, surface_provider))]
    pub fn create(
        vk_context: Arc<VkContext>,
        surface_provider: Arc<dyn SurfaceProvider>,
    ) -> Result<Self> {
        let surface = Self::create_surface(&vk_context, surface_provider)?;

        let vk_surface = Self {
            vk_context,

            surface,
        };

        info!("VkSurface created");

        Ok(vk_surface)
    }

    fn create_surface(
        vk_context: &VkContext,
        surface_provider: Arc<dyn SurfaceProvider>,
    ) -> Result<SurfaceKHR> {
        let (raw_display_handle, raw_window_handle) = surface_provider.handles();

        let surface = unsafe {
            create_surface(
                &vk_context.entry,
                &vk_context.instance,
                raw_display_handle,
                raw_window_handle,
                None,
            )
        }
        .context("create_surface")?;

        Ok(surface)
    }
}

impl Drop for VkSurface {
    fn drop(&mut self) {
        unsafe {
            self.vk_context
                .surface_loader
                .destroy_surface(self.surface, None);
        }
        info!("VkSurface destroyed");
    }
}
