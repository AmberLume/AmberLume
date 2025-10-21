use crate::render::vulkan::vk_context::VkContext;
use anyhow::Context;
use anyhow::Result;
use ash::vk;
use ash_window::create_surface;
use std::sync::Arc;
use tracing::{info, instrument};
use vk::SurfaceKHR;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;

pub struct VkSurface {
    vk_context: Arc<VkContext>,

    pub surface: SurfaceKHR,
}

impl VkSurface {
    #[instrument(level = "trace", skip(vk_context, window))]
    pub fn create(vk_context: Arc<VkContext>, window: &Window) -> Result<Self> {
        let surface = Self::create_surface(&vk_context, &window)?;

        let vk_surface = Self {
            vk_context,

            surface,
        };

        info!("VkSurface created");

        Ok(vk_surface)
    }

    fn create_surface(vk_context: &VkContext, window: &Window) -> Result<SurfaceKHR> {
        let raw_display = window.display_handle()?.as_raw();
        let raw_window_handle = window.window_handle()?.as_raw();
        let surface = unsafe {
            create_surface(
                &vk_context.entry,
                &vk_context.instance,
                raw_display,
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
