use crate::render::vulkan::physical_device_info::PhysicalDeviceInfo;
use crate::render::vulkan::queue::queue_families::QueueFamilies;
use crate::render::vulkan::queue::queues::Queues;
use crate::render::vulkan::vk_context::VkContext;
use crate::render::vulkan::vk_surface::VkSurface;
use anyhow::{Result, anyhow};
use ash::Device;
use ash::khr::{dynamic_rendering, swapchain};
use ash::vk::{
    DeviceCreateInfo, DeviceQueueCreateInfo, PhysicalDevice,
    PhysicalDeviceDynamicRenderingFeaturesKHR, PhysicalDeviceVulkan12Features,
};
use tracing::{info, instrument};

pub struct DeviceContext {
    pub device: Device,
    pub physical_device_info: PhysicalDeviceInfo,

    pub queue_families: QueueFamilies,
    pub queues: Queues,
}

impl DeviceContext {
    pub fn new(vk_context: &VkContext, vk_surface: &VkSurface) -> Result<Self> {
        let physical_device_info = vk_context
            .physical_devices
            .iter()
            .find(|physical_device| {
                physical_device
                    .is_suitable_for(&vk_context, &vk_surface)
                    .is_ok()
            })
            .cloned()
            .ok_or_else(|| anyhow!("No suitable device found"))?;

        let queue_families =
            QueueFamilies::find(&vk_context, vk_surface.surface, physical_device_info.handle)?;

        let device =
            Self::create_device(&vk_context, physical_device_info.handle, &queue_families)?;
        let queues = Queues::new(&device, &queue_families);

        Ok(Self {
            device,
            physical_device_info,

            queue_families,
            queues,
        })
    }

    #[instrument(level = "trace", skip_all)]
    fn create_device(
        vk_context: &VkContext,
        physical_device: PhysicalDevice,
        queue_families: &QueueFamilies,
    ) -> Result<Device> {
        let unique = queue_families.unique_families();
        let priorities = [1.0f32];
        let device_queue_create_info: Vec<_> = unique
            .iter()
            .map(|&i| {
                DeviceQueueCreateInfo::default()
                    .queue_family_index(i)
                    .queue_priorities(&priorities)
            })
            .collect();

        let extensions = [swapchain::NAME.as_ptr(), dynamic_rendering::NAME.as_ptr()];
        info!("Created device extensions: {:?}", extensions);

        let mut features_1_2 = PhysicalDeviceVulkan12Features::default()
            .buffer_device_address(true)
            .descriptor_indexing(true);

        let mut dynamic_rendering_features =
            PhysicalDeviceDynamicRenderingFeaturesKHR::default().dynamic_rendering(true);

        let device_create_info = DeviceCreateInfo::default()
            .queue_create_infos(&device_queue_create_info)
            .enabled_extension_names(&extensions)
            .push_next(&mut features_1_2)
            .push_next(&mut dynamic_rendering_features);

        let device = unsafe {
            vk_context
                .instance
                .create_device(physical_device, &device_create_info, None)?
        };

        info!("Logical device created");

        Ok(device)
    }

    pub fn destroy(&self) {
        unsafe {
            self.device.device_wait_idle().ok();
        }
        unsafe {
            self.device.destroy_device(None);
        }
    }
}
