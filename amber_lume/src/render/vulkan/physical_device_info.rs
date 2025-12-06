use crate::render::vulkan::surface::vulkan_surface::VulkanSurface;
use crate::render::vulkan::vulkan_context::VulkanContext;
use anyhow::{Context, Result, bail};
use ash::Instance;
use ash::khr::swapchain;
use ash::vk;
use ash::vk::{ExtensionProperties, PhysicalDevice, PhysicalDeviceFeatures, QueueFamilyProperties};
use std::ffi::CStr;
use tracing::debug;
use vk::{ImageUsageFlags, PhysicalDeviceProperties};

#[derive(Clone, Debug)]
pub struct PhysicalDeviceInfo {
    pub handle: PhysicalDevice,

    pub properties: PhysicalDeviceProperties,
    pub features: PhysicalDeviceFeatures,
    pub queue_family_properties: Vec<QueueFamilyProperties>,
    pub extension_properties: Vec<ExtensionProperties>,
}

impl PhysicalDeviceInfo {
    pub fn create(physical_device: PhysicalDevice, instance: &Instance) -> Result<Self> {
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let features = unsafe { instance.get_physical_device_features(physical_device) };
        let queue_family_properties =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        let extension_properties =
            unsafe { instance.enumerate_device_extension_properties(physical_device)? };

        Ok(Self {
            handle: physical_device,

            properties,
            features,
            queue_family_properties,
            extension_properties,
        })
    }

    pub fn create_all(instance: &Instance) -> Result<Vec<Self>> {
        let physical_devices = unsafe { instance.enumerate_physical_devices()? };

        let mut vec = Vec::with_capacity(physical_devices.len());

        for physical_device in physical_devices {
            let physical_device_info = Self::create(physical_device, &instance);

            vec.push(physical_device_info?);
        }

        Ok(vec)
    }

    pub fn is_suitable_for(
        &self,
        vk_context: &VulkanContext,
        vk_surface: &VulkanSurface,
    ) -> Result<()> {
        self.supports_present(&vk_context, &vk_surface)?;
        self.has_formats(&vk_context, &vk_surface)?;
        self.has_modes(&vk_context, &vk_surface)?;
        self.has_swapchain_extension()?;
        self.supports_color_attachment(&vk_context, &vk_surface)?;

        Ok(())
    }

    fn supports_present(
        &self,
        vk_context: &VulkanContext,
        vk_surface: &VulkanSurface,
    ) -> Result<()> {
        let queue_family_properties_vec = unsafe {
            vk_context
                .instance
                .get_physical_device_queue_family_properties(self.handle)
        };
        debug!("Queue family properties: {:?}", queue_family_properties_vec);
        let queue_family_properties_count = queue_family_properties_vec.len() as u32;

        let has_present = (0..queue_family_properties_count).any(|queue_family_index| unsafe {
            vk_context
                .surface_loader
                .get_physical_device_surface_support(
                    self.handle,
                    queue_family_index,
                    vk_surface.surface,
                )
                .unwrap_or(false)
        });

        if !has_present {
            bail!("No present-capable queue family for this surface");
        }

        Ok(())
    }

    fn has_formats(&self, vk_context: &VulkanContext, vk_surface: &VulkanSurface) -> Result<()> {
        let formats = unsafe {
            vk_context
                .surface_loader
                .get_physical_device_surface_formats(self.handle, vk_surface.surface)
        }
        .context("query surface formats")?;

        if formats.is_empty() {
            bail!("No supported formats")
        };

        Ok(())
    }

    fn has_modes(&self, vk_context: &VulkanContext, vk_surface: &VulkanSurface) -> Result<()> {
        let modes = unsafe {
            vk_context
                .surface_loader
                .get_physical_device_surface_present_modes(self.handle, vk_surface.surface)
        }
        .context("query surface modes")?;

        if modes.is_empty() {
            bail!("No supported modes")
        };

        Ok(())
    }

    fn has_swapchain_extension(&self) -> Result<()> {
        let presented = self
            .extension_properties
            .iter()
            .any(|extension_properties| unsafe {
                CStr::from_ptr(extension_properties.extension_name.as_ptr()) == swapchain::NAME
            });

        if !presented {
            bail!("Swapchain extension is not supported");
        }

        Ok(())
    }

    fn supports_color_attachment(
        &self,
        vk_context: &VulkanContext,
        vk_surface: &VulkanSurface,
    ) -> Result<()> {
        let surface_capabilities = unsafe {
            vk_context
                .surface_loader
                .get_physical_device_surface_capabilities(self.handle, vk_surface.surface)
        }
        .context("query surface capabilities")?;

        if !surface_capabilities
            .supported_usage_flags
            .contains(ImageUsageFlags::COLOR_ATTACHMENT)
        {
            bail!("Surface does not support COLOR_ATTACHMENT usage");
        }
        Ok(())
    }
}
