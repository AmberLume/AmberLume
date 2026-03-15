use crate::render::surface::render_surface::RenderSurface;
use crate::render::device::vulkan_context::VulkanContext;
use anyhow::{Result, bail};
use ash::vk;
use ash::vk::PhysicalDevice;
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
    pub present: QueueFamily,
    pub transfer: Option<QueueFamily>,
    pub compute: Option<QueueFamily>,
}

impl QueueFamilies {
    #[instrument(level = "trace", skip_all)]
    pub fn find(
        vulkan_context: &VulkanContext,
        render_surface: &RenderSurface,
        physical_device: PhysicalDevice,
    ) -> Result<Self> {
        let queue_family_properties = unsafe {
            vulkan_context
                .instance
                .get_physical_device_queue_family_properties(physical_device)
        };

        let surface_loader = &vulkan_context.surface_loader;

        let mut graphics = None;
        let mut present = None;
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
            let is_present = unsafe {
                surface_loader.get_physical_device_surface_support(
                    physical_device,
                    index,
                    render_surface.surface,
                )?
            };

            if is_graphics && graphics.is_none() {
                graphics = Some(queue_family);
            }

            if is_present && present.is_none() {
                present = Some(queue_family);
            }

            if is_transfer && !is_graphics && transfer.is_none() {
                transfer = Some(queue_family);
            }

            if is_compute && !is_graphics && compute.is_none() {
                compute = Some(queue_family);
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
        let mut unique = vec![self.graphics.index];

        if self.present.index != self.graphics.index {
            unique.push(self.present.index);
        }

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
