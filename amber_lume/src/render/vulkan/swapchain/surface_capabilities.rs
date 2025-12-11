use crate::render::vulkan::physical_device_info::PhysicalDeviceInfo;
use crate::render::vulkan::surface::vulkan_surface::VulkanSurface;
use crate::render::vulkan::vulkan_context::VulkanContext;
use anyhow::Result;
use ash::vk::SurfaceCapabilitiesKHR;
use tracing::info;

pub fn create_surface_capabilities(
    vulkan_context: &VulkanContext,
    vulkan_surface: &VulkanSurface,
    physical_device_info: &PhysicalDeviceInfo,
) -> Result<SurfaceCapabilitiesKHR> {
    let surface_capabilities = unsafe {
        vulkan_context
            .surface_loader
            .get_physical_device_surface_capabilities(
                physical_device_info.handle,
                vulkan_surface.surface,
            )?
    };

    info!("SurfaceCapabilities created: {:?}", surface_capabilities);

    Ok(surface_capabilities)
}
