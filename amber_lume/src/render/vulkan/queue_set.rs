use crate::render::vulkan::queue_families::QueueFamilies;
use ash::Device;
use ash::vk::Queue;
use tracing::debug;

pub struct QueueSet {
    pub graphics: Queue,
    pub present: Queue,
}

impl QueueSet {
    pub fn get(device: &Device, queue_families: &QueueFamilies) -> Self {
        let graphics = unsafe { device.get_device_queue(queue_families.graphics, 0) };
        let present = unsafe { device.get_device_queue(queue_families.present, 0) };

        debug!("Queue set created");

        Self { graphics, present }
    }
}
