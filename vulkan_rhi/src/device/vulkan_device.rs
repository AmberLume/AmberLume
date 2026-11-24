use crate::device::physical_device::PhysicalDevice;
use crate::instance::vulkan_instance::VulkanInstance;
use anyhow::Result;
use ash::khr::swapchain;
use ash::vk::{DeviceCreateInfo, DeviceQueueCreateInfo, PhysicalDeviceVulkan13Features};
use logger::info;
use std::ffi::c_char;

pub(crate) struct VulkanDevice {
    pub(crate) device: ash::Device,
    pub(crate) swapchain_loader: swapchain::Device,
}

impl VulkanDevice {
    pub(crate) fn create(instance: &VulkanInstance, physical_device: &PhysicalDevice) -> Result<Self> {
        let priorities = [1.0f32];
        let queue_create_infos = [DeviceQueueCreateInfo::default()
            .queue_family_index(physical_device.graphics_family)
            .queue_priorities(&priorities)];

        let extensions = PhysicalDevice::EXTENSIONS
            .iter()
            .map(|name| name.as_ptr())
            .collect::<Vec<*const c_char>>();

        let mut features_1_3 = PhysicalDeviceVulkan13Features::default()
            .dynamic_rendering(true)
            .synchronization2(true);

        let create_info = DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&extensions)
            .push_next(&mut features_1_3);

        let device = unsafe {
            instance
                .instance
                .create_device(physical_device.handle, &create_info, None)?
        };

        let swapchain_loader = swapchain::Device::new(&instance.instance, &device);

        info!("Device created: {}", physical_device.name);

        Ok(Self {
            device,
            swapchain_loader,
        })
    }

    pub(crate) fn wait_idle(&self) -> Result<()> {
        unsafe { self.device.device_wait_idle()? };

        Ok(())
    }

    pub(crate) fn destroy(self) -> Result<()> {
        let idle = self.wait_idle();

        unsafe { self.device.destroy_device(None) };

        info!("Device destroyed");

        idle
    }
}
