use crate::render::vulkan::renderer::command_recording::CommandRecording;
use anyhow::Result;
use ash::Device;
use ash::vk::{Fence, FenceCreateFlags, FenceCreateInfo, Semaphore, SemaphoreCreateInfo};
use tracing::info;
use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::vulkan::queue::queues::Queues;
use crate::render::vulkan::renderer::stats::raw_frame_stats::RawFrameStats;

pub struct FrameContext {
    pub fence: Fence,

    pub acquire_semaphore: Semaphore,

    pub command_recording: CommandRecording,

    pub raw_frame_stats: RawFrameStats,
}

impl FrameContext {
    pub fn create(
        device: &Device,
        queues: &Queues, 
        buffer_factory: &ManagedBufferFactory,
    ) -> Result<Self> {
        let fence_create_info = FenceCreateInfo::default().flags(FenceCreateFlags::SIGNALED);
        let fence = unsafe { device.create_fence(&fence_create_info, None)? };

        let semaphore_create_info = SemaphoreCreateInfo::default();
        let acquire_semaphore = unsafe { device.create_semaphore(&semaphore_create_info, None)? };

        let command_recording = CommandRecording::create(&device, queues.graphics_queue_family())?;

        let raw_frame_stats = RawFrameStats::new(&device, &buffer_factory);

        info!("FrameContext created");

        Ok(Self {
            fence,

            acquire_semaphore,

            command_recording,

            raw_frame_stats,
        })
    }

    pub fn destroy(
        self,
        device: &Device,
        buffer_factory: &ManagedBufferFactory,
    ) -> Result<()> {
        self.command_recording.destroy(&device)?;

        unsafe { device.destroy_semaphore(self.acquire_semaphore, None) };

        unsafe { device.destroy_fence(self.fence, None) };

        self.raw_frame_stats.destroy(&device, &buffer_factory)?;
        
        info!("FrameContext destroyed");

        Ok(())
    }
}
