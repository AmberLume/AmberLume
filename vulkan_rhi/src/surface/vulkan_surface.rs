use crate::device::physical_device::PhysicalDevice;
use crate::device::vulkan_device::VulkanDevice;
use crate::instance::vulkan_instance::VulkanInstance;
use crate::swapchain::surface_extent::SurfaceExtent;
use crate::swapchain::swapchain_config::SwapchainConfig;
use crate::swapchain::vulkan_swapchain::VulkanSwapchain;
use anyhow::{Result, anyhow};
use ash::khr::surface::Instance;
use ash::vk::{self, SurfaceKHR};
use ash_window::create_surface;
use explicit_rhi::{PresentSettings, Window, WindowSize};
use logger::info;

pub(crate) struct VulkanSurface {
    loader: Instance,
    physical_device: vk::PhysicalDevice,

    surface: SurfaceKHR,
    size: WindowSize,

    config: SwapchainConfig,
    swapchain: Option<VulkanSwapchain>,
}

impl VulkanSurface {
    pub fn create(
        instance: &VulkanInstance,
        physical_device: &PhysicalDevice,
        device: &VulkanDevice,
        window: Window,
        settings: &PresentSettings,
    ) -> Result<Self> {
        instance.check_surface(&window)?;

        let surface = unsafe {
            create_surface(
                &instance.entry,
                &instance.instance,
                window.display,
                window.window,
                None,
            )?
        };

        info!("Surface created");

        let loader = instance.surface_loader.clone();
        let size = window.size;

        let swapchain = physical_device
            .check_surface(&loader, surface)
            .and_then(|()| SwapchainConfig::select(&loader, physical_device.handle, surface, size, settings))
            .and_then(|config| VulkanSwapchain::create(None, device, surface, config));

        let swapchain = match swapchain {
            Ok(swapchain) => swapchain,
            Err(error) => {
                unsafe { loader.destroy_surface(surface, None) };

                info!("Surface destroyed");

                return Err(error);
            }
        };

        let physical_device = physical_device.handle;
        let config = swapchain.config;
        let swapchain = Some(swapchain);

        Ok(Self {
            loader,
            physical_device,

            surface,
            size,

            config,
            swapchain,
        })
    }

    pub fn resize(&mut self, device: &VulkanDevice, size: WindowSize) -> Result<()> {
        self.size = size;

        let capabilities = unsafe {
            self.loader
                .get_physical_device_surface_capabilities(self.physical_device, self.surface)?
        };

        let extent = SurfaceExtent::select(&capabilities, size);
        let pre_transform = capabilities.current_transform;

        let config = SwapchainConfig {
            extent,
            pre_transform,
            ..self.config
        };

        self.rebuild(device, config)
    }

    pub fn set_present_settings(&mut self, device: &VulkanDevice, settings: &PresentSettings) -> Result<()> {
        let config = SwapchainConfig::select(&self.loader, self.physical_device, self.surface, self.size, settings)?;

        self.rebuild(device, config)
    }

    pub fn destroy(self, device: &VulkanDevice) -> Result<()> {
        let idle = device.wait_idle();

        if let Some(swapchain) = self.swapchain {
            swapchain.destroy(device);
        }

        unsafe { self.loader.destroy_surface(self.surface, None) };

        info!("Surface destroyed");

        idle
    }

    fn rebuild(&mut self, device: &VulkanDevice, config: SwapchainConfig) -> Result<()> {
        let old = self.swapchain.as_ref().ok_or_else(|| {
            anyhow!(
                "The swapchain was lost in a failed rebuild, the window has to be attached again"
            )
        })?;

        let minimized = config.extent.width == 0 || config.extent.height == 0;

        if !minimized && config != old.config {
            device.wait_idle()?;

            match VulkanSwapchain::create(Some(old), device, self.surface, config) {
                Ok(swapchain) => {
                    if let Some(old) = self.swapchain.replace(swapchain) {
                        old.destroy(device);
                    }
                }
                Err(error) => {
                    if let Some(old) = self.swapchain.take() {
                        old.destroy(device);
                    }

                    return Err(error);
                }
            }
        }

        self.config = config;

        Ok(())
    }
}
