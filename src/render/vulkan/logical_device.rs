use crate::render::vulkan::instance_surface::InstanceSurface;
use crate::render::vulkan::queue_families::QueueFamilies;
use anyhow::Result;
use ash::khr::swapchain;
use ash::vk::PhysicalDevice;
use ash::{Device, vk};
use tracing::{info, instrument};
use vk::{DeviceCreateInfo, DeviceQueueCreateInfo};

pub struct LogicalDevice {
    pub device: Device,
    pub physical_device: PhysicalDevice,
}

impl LogicalDevice {
    #[instrument(level = "trace", skip_all)]
    pub fn create(
        inst: &InstanceSurface,
        physical_device: PhysicalDevice,
        queue_families: &QueueFamilies,
    ) -> Result<Self> {
        let unique = if queue_families.graphics == queue_families.present {
            vec![queue_families.graphics]
        } else {
            vec![queue_families.graphics, queue_families.present]
        };
        let priorities = [1.0f32];
        let device_queue_create_info: Vec<_> = unique
            .iter()
            .map(|&i| {
                DeviceQueueCreateInfo::default()
                    .queue_family_index(i)
                    .queue_priorities(&priorities)
            })
            .collect();

        let extensions = [swapchain::NAME.as_ptr()];
        info!("Created device extensions: {:?}", extensions);
        let device_create_info = DeviceCreateInfo::default()
            .queue_create_infos(&device_queue_create_info)
            .enabled_extension_names(&extensions);
        let device = unsafe {
            inst.instance
                .create_device(physical_device, &device_create_info, None)?
        };

        info!("Logical device created");

        let logical_device = LogicalDevice {
            device,
            physical_device,
        };

        Ok(logical_device)
    }

    pub fn destroy(&self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }
}
