use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::renderer::command_recording::CommandRecording;
use anyhow::Result;
use ash::vk::{Fence, FenceCreateFlags, FenceCreateInfo, Semaphore, SemaphoreCreateInfo};
use tracing::info;
use crate::render::vulkan::renderer::stats::raw_frame_stats::RawFrameStats;

pub struct FrameContext {
    pub fence: Fence,

    pub acquire_semaphore: Semaphore,

    pub command_recording: CommandRecording,

    pub raw_frame_stats: RawFrameStats,
}

impl FrameContext {
    pub fn create(device_context: &DeviceContext) -> Result<Self> {
        let device = &device_context.device;

        let fence_create_info = FenceCreateInfo::default().flags(FenceCreateFlags::SIGNALED);
        let fence = unsafe { device.create_fence(&fence_create_info, None)? };

        let semaphore_create_info = SemaphoreCreateInfo::default();
        let acquire_semaphore = unsafe { device.create_semaphore(&semaphore_create_info, None)? };

        let command_recording = CommandRecording::create(&device_context)?;

        let raw_frame_stats = RawFrameStats::new(&device_context);

        info!("FrameContext created");

        Ok(Self {
            fence,

            acquire_semaphore,

            command_recording,

            raw_frame_stats,
        })
    }

    pub fn destroy(&self, device_context: &DeviceContext) -> Result<()> {
        self.command_recording.destroy(&device_context)?;

        let device = &device_context.device;

        unsafe { device.destroy_semaphore(self.acquire_semaphore, None) };

        unsafe { device.destroy_fence(self.fence, None) };

        self.raw_frame_stats.destroy(&device_context);
        
        info!("FrameContext destroyed");

        Ok(())
    }
}
