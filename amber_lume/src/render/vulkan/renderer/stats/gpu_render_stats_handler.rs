use anyhow::Result;
use ash::Device;
use ash::vk::{AccessFlags, BufferUsageFlags, CommandBuffer, DependencyFlags, DeviceSize, MemoryBarrier, PipelineStageFlags};
use gpu_allocator::MemoryLocation;
use crate::render::vulkan::factories::buffer::managed_buffer::ManagedBuffer;
use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::vulkan::renderer::stats::gpu_render_stats::GpuRenderStats;
use crate::render::vulkan::renderer::stats::gpu_stage_measurement_recorder::GpuStageMeasurementRecorder;

pub struct GpuRenderStatsHandler {
    device: Device,

    pub buffer: ManagedBuffer,
    mapped_ptr: *const GpuRenderStats,

    pub stage_recorder: GpuStageMeasurementRecorder,
}

impl GpuRenderStatsHandler {
    pub fn create(
        device: Device,
        managed_buffer_factory: &ManagedBufferFactory,
        frames_in_flight: u32,
    ) -> Result<Self> {
        let buffer_size = (size_of::<GpuRenderStats>() as u32 * frames_in_flight) as DeviceSize;

        let buffer = managed_buffer_factory.create_managed_buffer(
            "render_stats",
            buffer_size,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuToCpu,
        )?;

        let mapped_ptr = buffer.mapped_ptr() as *const GpuRenderStats;

        let stage_recorder = GpuStageMeasurementRecorder::new(
            device.clone(),
            frames_in_flight,
        )?;

        Ok(Self {
            device,

            buffer,
            mapped_ptr,

            stage_recorder,
        })
    }

    pub fn reset(&self, command_buffer: CommandBuffer, frame_index: u32) {
        let stats_size = size_of::<GpuRenderStats>() as DeviceSize;
        let offset = stats_size * frame_index as DeviceSize;

        unsafe {
            self.device.cmd_fill_buffer(
                command_buffer,
                self.buffer.handle,
                offset,
                stats_size,
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

    pub fn read(&self, frame_index: usize) -> Result<GpuRenderStats> {
        Ok(unsafe { self.mapped_ptr.add(frame_index).read() })
    }

    pub fn collect(&self, command_buffer: CommandBuffer, frame_index: u32) {
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

        let offset = (size_of::<GpuRenderStats>() as u32 * frame_index) as DeviceSize;

        self.stage_recorder.copy_to_buffer(
            command_buffer,
            frame_index,
            &self.buffer,
            offset,
        )
    }

    pub fn destroy(
        self,
        buffer_factory: &ManagedBufferFactory,
    ) -> Result<()> {
        self.stage_recorder.destroy();

        buffer_factory.destroy_buffer(self.buffer)?;

        Ok(())
    }
}
