use crate::render::device::physical_device_info::PhysicalDeviceInfo;
use crate::render::surface::render_surface::RenderSurface;
use crate::render::device::vulkan_context::VulkanContext;
use anyhow::Result;
use ash::vk::SurfaceCapabilitiesKHR;
use tracing::info;

pub fn create_surface_capabilities(
    vulkan_context: &VulkanContext,
    render_surface: &RenderSurface,
    physical_device_info: &PhysicalDeviceInfo,
) -> Result<SurfaceCapabilitiesKHR> {
    let surface_capabilities = unsafe {
        vulkan_context
            .surface_loader
            .get_physical_device_surface_capabilities(
                physical_device_info.handle,
                render_surface.surface,
            )?
    };

    info!("SurfaceCapabilities created: {:?}", surface_capabilities);

    Ok(surface_capabilities)
}
