use crate::render::frame::command_recording::CommandRecording;
use anyhow::Result;
use ash::Device;
use ash::vk::{Fence, FenceCreateFlags, FenceCreateInfo, Semaphore, SemaphoreCreateInfo};
use tracing::info;
use gpu::Queues;

pub struct FrameContext {
    pub fence: Fence,

    pub acquire_semaphore: Semaphore,

    pub command_recording: CommandRecording,
}

impl FrameContext {
    pub fn create(
        device: &Device,
        queues: &Queues,
    ) -> Result<Self> {
        let fence_create_info = FenceCreateInfo::default().flags(FenceCreateFlags::SIGNALED);
        let fence = unsafe { device.create_fence(&fence_create_info, None)? };

        let semaphore_create_info = SemaphoreCreateInfo::default();
        let acquire_semaphore = unsafe { device.create_semaphore(&semaphore_create_info, None)? };

        let command_recording = CommandRecording::create(&device, queues.graphics_queue_family())?;

        info!("FrameContext created");

        Ok(Self {
            fence,

            acquire_semaphore,

            command_recording,
        })
    }

    pub fn destroy(
        self,
        device: &Device,
    ) -> Result<()> {
        self.command_recording.destroy(&device)?;

        unsafe { device.destroy_semaphore(self.acquire_semaphore, None) };

        unsafe { device.destroy_fence(self.fence, None) };

        info!("FrameContext destroyed");

        Ok(())
    }
}
