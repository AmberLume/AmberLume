use anyhow::Result;
use ash::Device;
use ash::vk::{Fence, FenceCreateFlags, FenceCreateInfo, Semaphore, SemaphoreCreateInfo};

pub struct FrameSync {
    pub fence: Fence,

    pub image_available: Semaphore,
    pub render_finished: Semaphore,
}

impl FrameSync {
    pub fn create(device: &Device) -> Result<Self> {
        let fence_create_info = FenceCreateInfo::default().flags(FenceCreateFlags::SIGNALED);
        let fence = unsafe { device.create_fence(&fence_create_info, None)? };

        let semaphore_create_info = SemaphoreCreateInfo::default();
        let image_available = unsafe { device.create_semaphore(&semaphore_create_info, None)? };
        let render_finished = unsafe { device.create_semaphore(&semaphore_create_info, None)? };

        Ok(Self {
            fence,

            image_available,
            render_finished,
        })
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_semaphore(self.image_available, None);
            device.destroy_semaphore(self.render_finished, None);
            device.destroy_fence(self.fence, None);
        }
    }
}
