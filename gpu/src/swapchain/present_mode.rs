use crate::device::physical_device_info::PhysicalDeviceInfo;
use crate::surface::render_surface::RenderSurface;
use crate::device::vulkan_context::VulkanContext;
use anyhow::Result;
use ash::vk::PresentModeKHR;
use tracing::info;

const FALLBACK_PRESENT_MODE: PresentModeKHR = PresentModeKHR::FIFO;

pub fn get_present_mode(
    vulkan_context: &VulkanContext,
    render_surface: &RenderSurface,
    physical_device_info: &PhysicalDeviceInfo,
    desired: PresentModeKHR,
) -> Result<PresentModeKHR> {
    let present_modes =
        query_present_modes(&vulkan_context, &render_surface, &physical_device_info)?;

    let present_mode = select_present_mode(&present_modes, desired);

    Ok(present_mode)
}

fn query_present_modes(
    vulkan_context: &VulkanContext,
    render_surface: &RenderSurface,
    physical_device_info: &PhysicalDeviceInfo,
) -> Result<Vec<PresentModeKHR>> {
    let present_modes = unsafe {
        vulkan_context
            .surface_loader
            .get_physical_device_surface_present_modes(
                physical_device_info.handle,
                render_surface.surface,
            )?
    };
    info!("Supported [PresentMode]: {:?}", present_modes);

    Ok(present_modes)
}

fn select_present_mode(
    present_modes: &[PresentModeKHR],
    desired: PresentModeKHR,
) -> PresentModeKHR {
    if !present_modes.contains(&desired) {
        info!(
            "PresentMode {:?} requested but unsupported, falling back to {:?}",
            desired, FALLBACK_PRESENT_MODE,
        );

        return FALLBACK_PRESENT_MODE;
    }

    info!("Selected PresentMode: {:?}", desired);

    desired
}
