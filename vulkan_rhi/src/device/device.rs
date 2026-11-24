use crate::device::device_features::DeviceFeatures;
use crate::device::physical_device::PhysicalDevice;
use crate::instance::vulkan_instance::VulkanInstance;
use anyhow::Result;
use ash::vk::{DeviceCreateInfo, DeviceQueueCreateInfo};
use explicit_rhi::DeviceSettings;
use logger::info;
use std::ffi::c_char;

pub struct Device {
    pub(crate) instance: VulkanInstance,
    pub(crate) device: ash::Device,
}

impl Device {
    fn create_device(
        instance: &VulkanInstance,
        physical_device: &PhysicalDevice,
    ) -> Result<ash::Device> {
        let priorities = [1.0f32];
        let queue_create_infos = [DeviceQueueCreateInfo::default()
            .queue_family_index(physical_device.graphics_family)
            .queue_priorities(&priorities)];

        let extensions = PhysicalDevice::EXTENSIONS
            .iter()
            .map(|name| name.as_ptr())
            .collect::<Vec<*const c_char>>();

        let mut features_1_3 = DeviceFeatures::REQUIRED.vulkan_1_3();

        let create_info = DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&extensions)
            .push_next(&mut features_1_3);

        let device = unsafe {
            instance
                .instance
                .create_device(physical_device.handle, &create_info, None)?
        };

        Ok(device)
    }
}

impl explicit_rhi::Device for Device {
    fn create(settings: DeviceSettings) -> Result<Self> {
        let instance = VulkanInstance::create(settings.validation)?;

        let physical_device = match PhysicalDevice::select(&instance.instance) {
            Ok(physical_device) => physical_device,
            Err(error) => {
                instance.destroy();

                return Err(error);
            }
        };

        let device = match Self::create_device(&instance, &physical_device) {
            Ok(device) => device,
            Err(error) => {
                instance.destroy();

                return Err(error);
            }
        };

        info!("Device created: {}", physical_device.name);

        Ok(Self { instance, device })
    }

    fn destroy(self) -> Result<()> {
        unsafe { self.device.device_wait_idle()? };
        unsafe { self.device.destroy_device(None) };

        info!("Device destroyed");

        self.instance.destroy();

        Ok(())
    }
}
