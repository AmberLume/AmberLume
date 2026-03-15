use crate::render::device::physical_device_info::PhysicalDeviceInfo;
use crate::render::surface::render_surface::RenderSurface;
use crate::render::device::vulkan_context::VulkanContext;
use anyhow::Result;
use ash::vk::{ColorSpaceKHR, Format, SurfaceFormatKHR};
use tracing::info;

pub fn get_surface_format(
    vulkan_context: &VulkanContext,
    render_surface: &RenderSurface,
    physical_device_info: &PhysicalDeviceInfo,
) -> Result<SurfaceFormatKHR> {
    let surface_formats =
        create_surface_formats(&vulkan_context, &render_surface, &physical_device_info)?;

    let surface_format = find_surface_format(
        &surface_formats,
        Format::B8G8R8A8_SRGB,
        ColorSpaceKHR::SRGB_NONLINEAR,
    )?;

    Ok(surface_format)
}

fn create_surface_formats(
    vulkan_context: &VulkanContext,
    render_surface: &RenderSurface,
    physical_device_info: &PhysicalDeviceInfo,
) -> Result<Vec<SurfaceFormatKHR>> {
    let surface_formats = unsafe {
        vulkan_context
            .surface_loader
            .get_physical_device_surface_formats(
                physical_device_info.handle,
                render_surface.surface,
            )?
    };

    info!("Supported [SurfaceFormat]: {:?}", surface_formats);

    Ok(surface_formats)
}

fn find_surface_format(
    surface_formats: &[SurfaceFormatKHR],
    desired_format: Format,
    desired_color_space: ColorSpaceKHR,
) -> Result<SurfaceFormatKHR> {
    let surface_format = surface_formats
        .iter()
        .copied()
        .find(|f| f.format == desired_format && f.color_space == desired_color_space)
        .unwrap_or(surface_formats[0]);

    info!("Selected SurfaceFormat: {:?}", surface_format);

    Ok(surface_format)
}
