use crate::render::vulkan::physical_device_info::PhysicalDeviceInfo;
use crate::render::vulkan::surface::vulkan_surface::VulkanSurface;
use crate::render::vulkan::vulkan_context::VulkanContext;
use anyhow::Result;
use ash::vk::PresentModeKHR;
use tracing::info;

pub fn get_present_mode(
    vulkan_context: &VulkanContext,
    vulkan_surface: &VulkanSurface,
    physical_device_info: &PhysicalDeviceInfo,
) -> Result<PresentModeKHR> {
    let surface_present_modes =
        create_surface_present_modes(&vulkan_context, &vulkan_surface, &physical_device_info)?;

    let present_mode = find_present_mode(
        surface_present_modes,
        PresentModeKHR::MAILBOX,
        PresentModeKHR::FIFO,
    )?;

    Ok(present_mode)
}

fn create_surface_present_modes(
    vulkan_context: &VulkanContext,
    vulkan_surface: &VulkanSurface,
    physical_device_info: &PhysicalDeviceInfo,
) -> Result<Vec<PresentModeKHR>> {
    let present_modes = unsafe {
        vulkan_context
            .surface_loader
            .get_physical_device_surface_present_modes(
                physical_device_info.handle,
                vulkan_surface.surface,
            )?
    };
    info!("Supported [PresentMode]: {:?}", present_modes);

    Ok(present_modes)
}

fn find_present_mode(
    present_modes: Vec<PresentModeKHR>,
    desired: PresentModeKHR,
    fallback: PresentModeKHR,
) -> Result<PresentModeKHR> {
    let present_mode = present_modes
        .into_iter()
        .find(|present_mode| *present_mode == desired)
        .unwrap_or(fallback);

    info!("Selected PresentMode: {:?}", present_mode);

    Ok(present_mode)
}
