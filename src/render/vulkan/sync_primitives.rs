use super::logical_device::LogicalDevice;
use anyhow::Result;
use ash::vk;
use vk::{Fence, FenceCreateFlags, FenceCreateInfo, Semaphore, SemaphoreCreateInfo};

pub struct SyncPrimitives {
    pub image_available: Semaphore,
    pub render_finished: Semaphore,
    pub in_flight: Fence,
}

impl SyncPrimitives {
    pub fn create(dev: &LogicalDevice) -> Result<Self> {
        let semaphore_create_info = SemaphoreCreateInfo::default();

        let fence_create_info = FenceCreateInfo::default().flags(FenceCreateFlags::SIGNALED);

        let sync_primitives = Self {
            image_available: unsafe { dev.device.create_semaphore(&semaphore_create_info, None)? },
            render_finished: unsafe { dev.device.create_semaphore(&semaphore_create_info, None)? },
            in_flight: unsafe { dev.device.create_fence(&fence_create_info, None)? },
        };

        Ok(sync_primitives)
    }
    pub fn destroy(&self, dev: &LogicalDevice) {
        unsafe {
            dev.device.destroy_fence(self.in_flight, None);
            dev.device.destroy_semaphore(self.render_finished, None);
            dev.device.destroy_semaphore(self.image_available, None);
        }
    }
}
