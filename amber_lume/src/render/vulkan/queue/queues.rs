use std::sync::{Arc, Mutex};
use crate::render::vulkan::queue::queue_families::QueueFamilies;
use ash::Device;
use ash::vk::Queue;

#[derive(Clone)]
pub struct QueueInfo {
    pub queue: Arc<Mutex<Queue>>,
    pub family: u32,
}

pub struct Queues {
    graphics: QueueInfo,
    present: QueueInfo,
    transfer: Option<QueueInfo>,
}

impl Queues {
    pub fn new(device: &Device, families: &QueueFamilies) -> Self {
        let graphics = {
            let queue = unsafe { device.get_device_queue(families.graphics, 0) };

            QueueInfo {
                queue: Arc::new(Mutex::new(queue)),
                family: families.graphics,
            }
        };

        let present = if families.present == families.graphics {
            graphics.clone()
        } else {
            let queue = unsafe { device.get_device_queue(families.present, 0) };

            QueueInfo {
                queue: Arc::new(Mutex::new(queue)),
                family: families.present,
            }
        };

        let transfer = families.transfer.map(|family| {
            if family == families.graphics {
                graphics.clone()
            } else if family == families.present {
                present.clone()
            } else {
                let queue = unsafe { device.get_device_queue(family, 0) };

                QueueInfo {
                    queue: Arc::new(Mutex::new(queue)),
                    family,
                }
            }
        });

        Self {
            graphics,
            present,
            transfer,
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
}
