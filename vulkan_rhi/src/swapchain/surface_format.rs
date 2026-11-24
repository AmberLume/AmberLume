use anyhow::{Result, anyhow};
use ash::khr::surface;
use ash::vk::{self, ColorSpaceKHR, PhysicalDevice, SurfaceFormatKHR, SurfaceKHR};
use explicit_rhi::{ColorSpace, Format, PresentFormat};
use surface::Instance;

pub(crate) struct SurfaceFormat;

impl SurfaceFormat {
    pub fn select(
        surface_loader: &Instance,
        physical_device: PhysicalDevice,
        surface: SurfaceKHR,
        formats: &[PresentFormat],
    ) -> Result<SurfaceFormatKHR> {
        let supported = unsafe {
            surface_loader.get_physical_device_surface_formats(physical_device, surface)?
        };

        formats
            .iter()
            .map(|&format| Self::vulkan_format(format))
            .find(|format| supported.contains(format))
            .ok_or_else(|| anyhow!("None of the formats {:?} is supported by the surface, supported: {:?}", formats, supported))
    }

    fn vulkan_format(format: PresentFormat) -> SurfaceFormatKHR {
        let color_space = match format.color_space {
            ColorSpace::SrgbNonlinear => ColorSpaceKHR::SRGB_NONLINEAR,
        };

        let format = match format.format {
            Format::Bgra8UnormSrgb => vk::Format::B8G8R8A8_SRGB,
            Format::Rgba8UnormSrgb => vk::Format::R8G8B8A8_SRGB,
        };

        SurfaceFormatKHR { format, color_space }
    }
}
