use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::renderer::command_recording::CommandRecording;
use anyhow::Result;
use ash::vk::{Fence, FenceCreateFlags, FenceCreateInfo, Semaphore, SemaphoreCreateInfo};
use tracing::info;

pub struct FrameContext {
    pub fence: Fence,

    pub image_available: Semaphore,
    pub render_finished: Semaphore,

    pub command_recording: CommandRecording,
}

impl FrameContext {
    pub fn create(device_context: &DeviceContext) -> Result<Self> {
        let device = &device_context.device;

        let fence_create_info = FenceCreateInfo::default().flags(FenceCreateFlags::SIGNALED);
        let fence = unsafe { device.create_fence(&fence_create_info, None)? };

        let semaphore_create_info = SemaphoreCreateInfo::default();
        let image_available = unsafe { device.create_semaphore(&semaphore_create_info, None)? };
        let render_finished = unsafe { device.create_semaphore(&semaphore_create_info, None)? };

        let command_recording = CommandRecording::create(&device_context)?;

        info!("FrameContext created");

        Ok(Self {
            fence,

            image_available,
            render_finished,

            command_recording,
        })
    }

    pub fn destroy(&self, device_context: &DeviceContext) -> Result<()> {
        self.command_recording.destroy(&device_context)?;

        let device = &device_context.device;

        unsafe { device.destroy_semaphore(self.image_available, None) };
        unsafe { device.destroy_semaphore(self.render_finished, None) };

        unsafe { device.destroy_fence(self.fence, None) };

        info!("FrameContext destroyed");

        Ok(())
    }
}
