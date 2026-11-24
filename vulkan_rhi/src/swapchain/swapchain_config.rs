use crate::swapchain::surface_composite_alpha::SurfaceCompositeAlpha;
use crate::swapchain::surface_extent::SurfaceExtent;
use crate::swapchain::surface_format::SurfaceFormat;
use crate::swapchain::surface_image_count::SurfaceImageCount;
use crate::swapchain::surface_image_usage::SurfaceImageUsage;
use crate::swapchain::surface_present_mode::SurfacePresentMode;
use anyhow::Result;
use ash::khr::surface::Instance;
use ash::vk::{
    CompositeAlphaFlagsKHR, Extent2D, ImageUsageFlags, PhysicalDevice, PresentModeKHR, SurfaceFormatKHR, SurfaceKHR,
    SurfaceTransformFlagsKHR,
};
use explicit_rhi::{PresentSettings, WindowSize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SwapchainConfig {
    pub format: SurfaceFormatKHR,
    pub present_mode: PresentModeKHR,
    pub composite_alpha: CompositeAlphaFlagsKHR,

    pub extent: Extent2D,
    pub image_count: u32,
    pub usage: ImageUsageFlags,
    pub pre_transform: SurfaceTransformFlagsKHR,
}

impl SwapchainConfig {
    pub fn select(
        loader: &Instance,
        physical_device: PhysicalDevice,
        surface: SurfaceKHR,
        size: WindowSize,
        settings: &PresentSettings,
    ) -> Result<Self> {
        let capabilities = unsafe {
            loader.get_physical_device_surface_capabilities(physical_device, surface)?
        };

        let format = SurfaceFormat::select(loader, physical_device, surface, &settings.formats)?;
        let present_mode = SurfacePresentMode::select(loader, physical_device, surface, &settings.present_modes)?;
        let composite_alpha = SurfaceCompositeAlpha::select(&capabilities, &settings.composite_alphas)?;

        let extent = SurfaceExtent::select(&capabilities, size);
        let image_count = SurfaceImageCount::select(&capabilities, settings.image_count);
        let usage = SurfaceImageUsage::select(&capabilities, settings.usage)?;
        let pre_transform = capabilities.current_transform;

        Ok(Self {
            format,
            present_mode,
            composite_alpha,

            extent,
            image_count,
            usage,
            pre_transform,
        })
    }
}
