use crate::render::vulkan::queue::queue_families::QueueFamilies;
use ash::Device;
use ash::vk::Queue;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum QueueType {
    Graphics,
    Present,
    Transfer,
    Compute,
}

#[derive(Clone, Copy)]
pub struct QueueInfo {
    pub queue: Queue,
    pub family: u32,
}

pub struct Queues {
    graphics: QueueInfo,
    present: QueueInfo,
    transfer: Option<QueueInfo>,
    compute: Option<QueueInfo>,
}

impl Queues {
    pub fn new(device: &Device, families: &QueueFamilies) -> Self {
        let graphics = QueueInfo {
            queue: unsafe { device.get_device_queue(families.graphics, 0) },
            family: families.graphics,
        };

        let present = QueueInfo {
            queue: unsafe { device.get_device_queue(families.present, 0) },
            family: families.present,
        };

        let transfer = families.transfer.map(|family| QueueInfo {
            queue: unsafe { device.get_device_queue(family, 0) },
            family,
        });

        let compute = families.compute.map(|family| QueueInfo {
            queue: unsafe { device.get_device_queue(family, 0) },
            family,
        });

        Self {
            graphics,
            present,
            transfer,
            compute,
        }
    }

    pub fn graphics(&self) -> &QueueInfo {
        &self.graphics
    }

    pub fn present(&self) -> &QueueInfo {
        &self.present
    }

    pub fn transfer(&self) -> &QueueInfo {
        self.transfer.as_ref().unwrap_or(&self.graphics)
    }

    pub fn compute(&self) -> &QueueInfo {
        self.compute.as_ref().unwrap_or(&self.graphics)
    }
}
