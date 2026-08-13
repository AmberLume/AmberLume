use crate::device::physical_device_info::PhysicalDeviceInfo;
use crate::surface::render_surface::RenderSurface;
use crate::device::vulkan_context::VulkanContext;
use anyhow::{anyhow, Result};
use ash::vk::{ColorSpaceKHR, Format, SurfaceFormatKHR};
use tracing::info;

pub const HDR_FORMAT: Format = Format::R16G16B16A16_SFLOAT;
const HDR_COLOR_SPACE: ColorSpaceKHR = ColorSpaceKHR::EXTENDED_SRGB_LINEAR_EXT;

pub fn get_surface_format(
    vulkan_context: &VulkanContext,
    render_surface: &RenderSurface,
    physical_device_info: &PhysicalDeviceInfo,
    hdr: bool,
) -> Result<SurfaceFormatKHR> {
    let surface_formats =
        query_surface_formats(&vulkan_context, &render_surface, &physical_device_info)?;

    let surface_format = select_surface_format(&surface_formats, hdr)?;

    Ok(surface_format)
}

pub(crate) fn query_surface_formats(
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

pub(crate) fn log_hdr_support(surface_formats: &[SurfaceFormatKHR]) {
    let hdr_color_spaces = surface_formats
        .iter()
        .map(|f| f.color_space)
        .filter(|c| {
            matches!(
                *c,
                ColorSpaceKHR::EXTENDED_SRGB_LINEAR_EXT
                    | ColorSpaceKHR::EXTENDED_SRGB_NONLINEAR_EXT
                    | ColorSpaceKHR::HDR10_ST2084_EXT
                    | ColorSpaceKHR::HDR10_HLG_EXT
                    | ColorSpaceKHR::BT2020_LINEAR_EXT
            )
        })
        .collect::<Vec<_>>();
    info!("Supported HDR color spaces: {:?}", hdr_color_spaces);
    info!("HDR (scRGB) supported: {}", surface_supports_hdr(surface_formats));
}

pub(crate) fn surface_supports_hdr(surface_formats: &[SurfaceFormatKHR]) -> bool {
    surface_formats
        .iter()
        .any(|f| f.format == HDR_FORMAT && f.color_space == HDR_COLOR_SPACE)
}

fn select_surface_format(
    surface_formats: &[SurfaceFormatKHR],
    hdr: bool,
) -> Result<SurfaceFormatKHR> {
    if hdr {
        if let Some(format) = surface_formats
            .iter()
            .copied()
            .find(|f| f.format == HDR_FORMAT && f.color_space == HDR_COLOR_SPACE)
        {
            info!("Selected SurfaceFormat: {:?}", format);

            return Ok(format);
        }

        info!("HDR requested but scRGB unavailable, falling back to sRGB");
    }

    const PREFERRED: &[Format] = &[
        Format::B8G8R8A8_SRGB,
        Format::R8G8B8A8_SRGB,
    ];

    let surface_format = PREFERRED
        .iter()
        .find_map(|&desired| {
            surface_formats.iter().copied().find(|f| {
                f.format == desired && f.color_space == ColorSpaceKHR::SRGB_NONLINEAR
            })
        })
        .or_else(|| {
            surface_formats
                .iter()
                .copied()
                .find(|f| f.color_space == ColorSpaceKHR::SRGB_NONLINEAR)
        })
        .ok_or_else(|| {
            anyhow!(
                "No sRGB swapchain format available. Available: {:?}",
                surface_formats,
            )
        })?;

    info!("Selected SurfaceFormat: {:?}", surface_format);

    Ok(surface_format)
}
