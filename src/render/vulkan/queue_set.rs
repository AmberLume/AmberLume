use crate::render::vulkan::logical_device::LogicalDevice;
use crate::render::vulkan::queue_families::QueueFamilies;
use ash::vk::Queue;
use tracing::debug;

pub struct QueueSet {
    pub graphics: Queue,
    pub present: Queue,
}

impl QueueSet {
    pub fn get(logical_device: &LogicalDevice, qf: &QueueFamilies) -> Self {
        let graphics = unsafe { logical_device.device.get_device_queue(qf.graphics, 0) };
        let present = unsafe { logical_device.device.get_device_queue(qf.present, 0) };

        debug!("Queue set created");

        Self { graphics, present }
    }
}
