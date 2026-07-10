use crate::render::surface::render_surface::RenderSurface;
use crate::render::device::vulkan_context::VulkanContext;
use anyhow::{Context, Result, bail};
use ash::Instance;
use ash::khr::{acceleration_structure, deferred_host_operations, ray_query, swapchain};
use ash::vk;
use ash::vk::{ExtensionProperties, PhysicalDevice, PhysicalDeviceFeatures};
use std::ffi::CStr;
use tracing::debug;
use vk::ImageUsageFlags;

#[derive(Clone, Debug)]
pub struct PhysicalDeviceInfo {
    pub handle: PhysicalDevice,

    pub features: PhysicalDeviceFeatures,
    pub extension_properties: Vec<ExtensionProperties>,
    pub timestamp_period: f32,
}

impl PhysicalDeviceInfo {
    pub fn create(physical_device: PhysicalDevice, instance: &Instance) -> Result<Self> {
        let features = unsafe { instance.get_physical_device_features(physical_device) };
        let extension_properties = unsafe { instance.enumerate_device_extension_properties(physical_device)? };

        let device_properties = unsafe { instance.get_physical_device_properties(physical_device) };

        Ok(Self {
            handle: physical_device,

            features,
            extension_properties,
            timestamp_period: device_properties.limits.timestamp_period,
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

    pub fn is_suitable(&self) -> Result<()> {
        self.has_swapchain_extension()?;

        Ok(())
    }

    pub fn validate_surface(
        &self,
        vulkan_context: &VulkanContext,
        render_surface: &RenderSurface,
    ) -> Result<()> {
        self.supports_present(vulkan_context, render_surface)?;
        self.has_formats(vulkan_context, render_surface)?;
        self.has_modes(vulkan_context, render_surface)?;
        self.supports_color_attachment(vulkan_context, render_surface)?;

        Ok(())
    }

    fn supports_present(
        &self,
        vulkan_context: &VulkanContext,
        render_surface: &RenderSurface,
    ) -> Result<()> {
        let queue_family_properties_vec = unsafe {
            vulkan_context
                .instance
                .get_physical_device_queue_family_properties(self.handle)
        };
        debug!("Queue family properties: {:?}", queue_family_properties_vec);
        let queue_family_properties_count = queue_family_properties_vec.len() as u32;

        let has_present = (0..queue_family_properties_count).any(|queue_family_index| unsafe {
            vulkan_context
                .surface_loader
                .get_physical_device_surface_support(
                    self.handle,
                    queue_family_index,
                    render_surface.surface,
                )
                .unwrap_or(false)
        });

        if !has_present {
            bail!("No present-capable queue family for this surface");
        }

        Ok(())
    }

    fn has_formats(
        &self,
        vulkan_context: &VulkanContext,
        render_surface: &RenderSurface,
    ) -> Result<()> {
        let formats = unsafe {
            vulkan_context
                .surface_loader
                .get_physical_device_surface_formats(self.handle, render_surface.surface)
        }
        .context("query surface formats")?;

        if formats.is_empty() {
            bail!("No supported formats")
        };

        Ok(())
    }

    fn has_modes(
        &self,
        vulkan_context: &VulkanContext,
        render_surface: &RenderSurface,
    ) -> Result<()> {
        let modes = unsafe {
            vulkan_context
                .surface_loader
                .get_physical_device_surface_present_modes(self.handle, render_surface.surface)
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

    pub fn supports_ray_tracing(&self) -> bool {
        [
            acceleration_structure::NAME,
            deferred_host_operations::NAME,
            ray_query::NAME,
        ]
        .iter()
        .all(|&name| {
            self.extension_properties.iter().any(|extension_properties| unsafe {
                CStr::from_ptr(extension_properties.extension_name.as_ptr()) == name
            })
        })
    }

    fn supports_color_attachment(
        &self,
        vulkan_context: &VulkanContext,
        render_surface: &RenderSurface,
    ) -> Result<()> {
        let surface_capabilities = unsafe {
            vulkan_context
                .surface_loader
                .get_physical_device_surface_capabilities(self.handle, render_surface.surface)
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
