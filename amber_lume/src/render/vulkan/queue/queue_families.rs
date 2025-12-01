use crate::render::vulkan::vk_context::VkContext;
use anyhow::{Result, bail};
use ash::vk;
use ash::vk::{PhysicalDevice, SurfaceKHR};
use tracing::{info, instrument};
use vk::QueueFlags;

#[derive(Clone, Copy, Debug)]
pub struct QueueFamilies {
    pub graphics: u32,
    pub present: u32,
    pub transfer: Option<u32>,
    pub compute: Option<u32>,
}

impl QueueFamilies {
    #[instrument(level = "trace", skip_all)]
    pub fn find(
        vk_context: &VkContext,
        surface: SurfaceKHR,
        physical_device: PhysicalDevice,
    ) -> Result<Self> {
        let queue_family_properties = unsafe {
            vk_context
                .instance
                .get_physical_device_queue_family_properties(physical_device)
        };

        let surface_loader = &vk_context.surface_loader;

        let mut graphics = None;
        let mut present = None;
        let mut transfer = None;
        let mut compute = None;

        info!("Searching for graphics queue families");
        for (index, properties) in queue_family_properties.iter().enumerate() {
            let index = index as u32;

            // Graphics
            if properties.queue_flags.contains(QueueFlags::GRAPHICS) && graphics.is_none() {
                graphics = Some(index);
            }

            // Present
            let present_support = unsafe {
                surface_loader.get_physical_device_surface_support(
                    physical_device,
                    index,
                    surface,
                )?
            };
            if present_support && present.is_none() {
                present = Some(index);
            }

            // Dedicated transfer
            if properties.queue_flags.contains(QueueFlags::TRANSFER)
                && !properties.queue_flags.contains(QueueFlags::GRAPHICS)
                && transfer.is_none()
            {
                transfer = Some(index);
            }

            // Dedicated compute
            if properties.queue_flags.contains(QueueFlags::COMPUTE)
                && !properties.queue_flags.contains(QueueFlags::GRAPHICS)
                && compute.is_none()
            {
                compute = Some(index);
            }
        }

        if graphics.is_none() {
            bail!("No graphics queue family")
        }
        if present.is_none() {
            bail!("No present queue family")
        }

        let queues = Self {
            graphics: graphics.unwrap(),
            present: present.unwrap(),
            transfer,
            compute,
        };

        info!("Found QueueFamilies: {:?}", queues);

        Ok(queues)
    }

    pub fn unique_families(&self) -> Vec<u32> {
        let mut unique = vec![self.graphics];

        if self.present != self.graphics {
            unique.push(self.present);
        }

        if let Some(t) = self.transfer {
            if !unique.contains(&t) {
                unique.push(t);
            }
        }
        if let Some(c) = self.compute {
            if !unique.contains(&c) {
                unique.push(c);
            }
        }

        unique
    }
}
