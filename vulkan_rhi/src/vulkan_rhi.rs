use crate::device::physical_device::PhysicalDevice;
use crate::device::vulkan_device::VulkanDevice;
use crate::instance::vulkan_instance::VulkanInstance;
use crate::surface::vulkan_surface::VulkanSurface;
use anyhow::{Result, anyhow, bail};
use explicit_rhi::{ExplicitRhi, ExplicitRhiSettings, PresentSettings, Window, WindowSize};

pub struct VulkanRhi {
    instance: VulkanInstance,
    physical_device: PhysicalDevice,
    device: VulkanDevice,

    present_settings: PresentSettings,

    surface: Option<VulkanSurface>,
}

impl ExplicitRhi for VulkanRhi {
    fn create(settings: ExplicitRhiSettings) -> Result<Self> {
        let instance = VulkanInstance::create(settings.validation)?;

        let physical_device = match PhysicalDevice::select(&instance.instance) {
            Ok(physical_device) => physical_device,
            Err(error) => {
                instance.destroy();

                return Err(error);
            }
        };

        let device = match VulkanDevice::create(&instance, &physical_device) {
            Ok(device) => device,
            Err(error) => {
                instance.destroy();

                return Err(error);
            }
        };

        let present_settings = PresentSettings::default();

        Ok(Self {
            instance,
            physical_device,
            device,

            present_settings,

            surface: None,
        })
    }

    fn attach(&mut self, window: Window) -> Result<()> {
        if self.surface.is_some() {
            bail!("A window is already attached");
        }

        let surface = VulkanSurface::create(
            &self.instance,
            &self.physical_device,
            &self.device,
            window,
            &self.present_settings,
        )?;

        self.surface = Some(surface);

        Ok(())
    }

    fn detach(&mut self) -> Result<()> {
        let surface = self.surface.take().ok_or_else(|| anyhow!("No window to detach"))?;

        surface.destroy(&self.device)
    }

    fn resize(&mut self, size: WindowSize) -> Result<()> {
        match self.surface.as_mut() {
            Some(surface) => surface.resize(&self.device, size),
            None => Ok(()),
        }
    }

    fn set_present_settings(&mut self, settings: PresentSettings) -> Result<()> {
        if let Some(surface) = self.surface.as_mut() {
            surface.set_present_settings(&self.device, &settings)?;
        }

        self.present_settings = settings;

        Ok(())
    }

    fn destroy(self) -> Result<()> {
        let surface = match self.surface {
            Some(surface) => surface.destroy(&self.device),
            None => Ok(()),
        };

        let device = self.device.destroy();

        self.instance.destroy();

        surface.and(device)
    }
}
