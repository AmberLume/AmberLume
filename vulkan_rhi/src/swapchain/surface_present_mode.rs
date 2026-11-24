use anyhow::{Result, anyhow};
use ash::khr::surface;
use ash::vk::{PhysicalDevice, PresentModeKHR, SurfaceKHR};
use explicit_rhi::PresentMode;
use surface::Instance;

pub(crate) struct SurfacePresentMode;

impl SurfacePresentMode {
    pub fn select(
        surface_loader: &Instance,
        physical_device: PhysicalDevice,
        surface: SurfaceKHR,
        present_modes: &[PresentMode],
    ) -> Result<PresentModeKHR> {
        let supported = unsafe {
            surface_loader.get_physical_device_surface_present_modes(physical_device, surface)?
        };

        present_modes
            .iter()
            .map(|&present_mode| Self::vulkan_present_mode(present_mode))
            .find(|present_mode| supported.contains(present_mode))
            .ok_or_else(|| {
                anyhow!("None of the present modes {:?} is supported by the surface, supported: {:?}", present_modes, supported)
            })
    }

    fn vulkan_present_mode(present_mode: PresentMode) -> PresentModeKHR {
        match present_mode {
            PresentMode::Fifo => PresentModeKHR::FIFO,
            PresentMode::Mailbox => PresentModeKHR::MAILBOX,
            PresentMode::Immediate => PresentModeKHR::IMMEDIATE,
        }
    }
}
