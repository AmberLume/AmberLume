use std::sync::Arc;
use anyhow::Result;
use ash::Device;
use ash::vk::{AccessFlags, CommandBuffer, DependencyFlags, MemoryBarrier, PipelineStageFlags};
use crate::ids::FrameIndex;
use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::renderer::stats::gpu_render_stats::GpuRenderStats;
use crate::render::vulkan::renderer::stats::gpu_stage_measurement_recorder::GpuStageMeasurementRecorder;

pub struct GpuRenderStatsHandler {
    device: Device,

    buffer_manager: Arc<BufferManager>,

    pub stage_recorder: GpuStageMeasurementRecorder,
}

impl GpuRenderStatsHandler {
    pub fn create(
        device: Device,
        buffer_manager: Arc<BufferManager>,
        frames_in_flight: u32,
    ) -> Result<Self> {
        let stage_recorder = GpuStageMeasurementRecorder::new(
            device.clone(),
            frames_in_flight,
        )?;

        Ok(Self {
            device,

            buffer_manager: buffer_manager.clone(),

            stage_recorder,
        })
    }

    pub fn reset(&self, command_buffer: CommandBuffer, frame_index: FrameIndex) {
        let buffer_view = self.buffer_manager.render_stats_buffer.frame(frame_index);

        unsafe {
            self.device.cmd_fill_buffer(
                command_buffer,
                buffer_view.get().handle(),
                buffer_view.get().offset(),
                buffer_view.item_size(),
                0,
            )
        }

        self.stage_recorder.reset(command_buffer, frame_index);

        let barrier = MemoryBarrier::default()
            .src_access_mask(AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE);

        unsafe {
            self.device.cmd_pipeline_barrier(
                command_buffer,
                PipelineStageFlags::TRANSFER,
                PipelineStageFlags::COMPUTE_SHADER,
                DependencyFlags::empty(),
                &[barrier],
                &[],
                &[],
            );
        }
    }

    pub fn read(&self, frame_index: FrameIndex) -> Result<GpuRenderStats> {
        let buffer_view = self.buffer_manager.render_stats_buffer.frame(frame_index);

        let mapped_ptr = buffer_view.get().mapped_ptr() as *mut GpuRenderStats;

        Ok(unsafe { mapped_ptr.read() })
    }

    pub fn collect(&self, command_buffer: CommandBuffer, frame_index: FrameIndex) {
        let barrier = MemoryBarrier::default()
            .src_access_mask(AccessFlags::SHADER_WRITE)
            .dst_access_mask(AccessFlags::TRANSFER_WRITE);

        unsafe {
            self.device.cmd_pipeline_barrier(
                command_buffer,
                PipelineStageFlags::COMPUTE_SHADER,
                PipelineStageFlags::TRANSFER,
                DependencyFlags::empty(),
                &[barrier],
                &[],
                &[],
            );
        }

        self.stage_recorder.copy_to_buffer(
            command_buffer,
            frame_index,
            &self.buffer_manager.render_stats_buffer.frame(frame_index).get(),
        )
    }

    pub fn destroy(
        self,
    ) -> Result<()> {
        self.stage_recorder.destroy();

        Ok(())
    }
}
