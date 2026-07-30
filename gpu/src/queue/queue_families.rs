use crate::device::vulkan_context::VulkanContext;
use anyhow::{Result, bail};
use ash::vk;
use ash::vk::{PhysicalDevice, SurfaceKHR};
use tracing::{info, instrument};
use vk::QueueFlags;

#[derive(Clone, Copy, Debug)]
pub struct QueueFamily {
    pub index: u32,
    pub queue_count: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct QueueFamilies {
    pub graphics: QueueFamily,
    pub transfer: Option<QueueFamily>,
    pub compute: Option<QueueFamily>,
}

impl QueueFamilies {
    #[instrument(level = "trace", skip_all)]
    pub fn find(
        vulkan_context: &VulkanContext,
        physical_device: PhysicalDevice,
    ) -> Result<Self> {
        let queue_family_properties = unsafe {
            vulkan_context
                .instance
                .get_physical_device_queue_family_properties(physical_device)
        };

        let mut graphics = None;
        let mut transfer = None;
        let mut compute = None;

        info!("Searching for graphics queue families");
        for (index, properties) in queue_family_properties.iter().enumerate() {
            let index = index as u32;
            let queue_count = properties.queue_count;

            let queue_family = QueueFamily { index, queue_count };

            let is_graphics = properties.queue_flags.contains(QueueFlags::GRAPHICS);
            let is_transfer = properties.queue_flags.contains(QueueFlags::TRANSFER);
            let is_compute = properties.queue_flags.contains(QueueFlags::COMPUTE);

            if is_graphics && graphics.is_none() {
                graphics = Some(queue_family);
            }

            if is_transfer && !is_graphics && transfer.is_none() {
                transfer = Some(queue_family);
            }

            if is_compute && !is_graphics && compute.is_none() {
                compute = Some(queue_family);
            }
        }

        let Some(graphics) = graphics else {
            bail!("No graphics queue family");
        };

        let queues = Self {
            graphics,
            transfer,
            compute,
        };

        info!("Found QueueFamilies: {:?}", queues);

        Ok(queues)
    }

    pub fn find_present(
        vulkan_context: &VulkanContext,
        physical_device: PhysicalDevice,
        surface: SurfaceKHR,
    ) -> Result<QueueFamily> {
        let queue_family_properties = unsafe {
            vulkan_context
                .instance
                .get_physical_device_queue_family_properties(physical_device)
        };

        for (index, properties) in queue_family_properties.iter().enumerate() {
            let index = index as u32;
            let supports_present = unsafe {
                vulkan_context.surface_loader.get_physical_device_surface_support(
                    physical_device,
                    index,
                    surface,
                )?
            };

            if supports_present {
                let queue_family = QueueFamily {
                    index,
                    queue_count: properties.queue_count,
                };
                
                info!("Found present queue family: {:?}", queue_family);
                
                return Ok(queue_family);
            }
        }

        bail!("No present-capable queue family for this surface");
    }

    pub fn unique_families(&self) -> Vec<u32> {
        let mut unique = vec![self.graphics.index];

        if let Some(transfer) = self.transfer {
            if !unique.contains(&transfer.index) {
                unique.push(transfer.index);
            }
        }
        if let Some(compute) = self.compute {
            if !unique.contains(&compute.index) {
                unique.push(compute.index);
            }
        }

        unique
    }
}
