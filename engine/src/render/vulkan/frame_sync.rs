use crate::render::vulkan::command_recording::CommandRecording;
use crate::render::vulkan::queue_families::QueueFamilies;
use anyhow::Result;
use ash::Device;
use ash::vk::{Fence, FenceCreateFlags, FenceCreateInfo, Semaphore, SemaphoreCreateInfo};

pub struct FrameSync {
    pub fence: Fence,

    pub image_available: Semaphore,
    pub render_finished: Semaphore,

    pub command_recording: CommandRecording,
}

impl FrameSync {
    pub fn create(device: &Device, queue_families: &QueueFamilies) -> Result<Self> {
        let fence_create_info = FenceCreateInfo::default().flags(FenceCreateFlags::SIGNALED);
        let fence = unsafe { device.create_fence(&fence_create_info, None)? };

        let semaphore_create_info = SemaphoreCreateInfo::default();
        let image_available = unsafe { device.create_semaphore(&semaphore_create_info, None)? };
        let render_finished = unsafe { device.create_semaphore(&semaphore_create_info, None)? };

        let command_recording =
            CommandRecording::create(&device, queue_families.graphics, [0.08, 0.10, 0.15, 1.0])?;

        Ok(Self {
            fence,

            image_available,
            render_finished,

            command_recording,
        })
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            self.command_recording.destroy(device);

            device.destroy_semaphore(self.image_available, None);
            device.destroy_semaphore(self.render_finished, None);

            device.destroy_fence(self.fence, None);
        }
    }
}
