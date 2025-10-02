use super::instance_surface::InstanceSurface;
use anyhow::{Result, anyhow};
use ash::vk;
use tracing::{instrument, trace};
use vk::PhysicalDevice;

pub struct PhysicalDeviceChoice {
    pub device: PhysicalDevice,
}

impl PhysicalDeviceChoice {
    #[instrument(level = "trace", skip_all)]
    pub fn pick(instance_surface: &InstanceSurface) -> Result<Self> {
        let physical_devices = unsafe { instance_surface.instance.enumerate_physical_devices()? };

        let chosen = physical_devices
            .into_iter()
            .find(|&physical_device| {
                let formats = unsafe {
                    instance_surface
                        .surface_loader
                        .get_physical_device_surface_formats(
                            physical_device,
                            instance_surface.surface,
                        )
                }
                .ok();
                let modes = unsafe {
                    instance_surface
                        .surface_loader
                        .get_physical_device_surface_present_modes(
                            physical_device,
                            instance_surface.surface,
                        )
                }
                .ok();

                formats.map(|f| !f.is_empty()).unwrap_or(false)
                    && modes.map(|m| !m.is_empty()).unwrap_or(false)
            })
            .ok_or_else(|| anyhow!("No suitable physical device"))?;

        trace!("Chosen device: {:?}", chosen);

        let physical_device_choice = Self { device: chosen };

        Ok(physical_device_choice)
    }
}
