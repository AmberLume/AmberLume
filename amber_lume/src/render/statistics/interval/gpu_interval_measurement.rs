use std::slice::from_raw_parts;
use crate::render::factories::query_pool::query_pool::ManagedQueryPool;
use crate::render::factories::query_pool::query_pool_factory::QueryPoolFactory;
use anyhow::Result;
use ash::vk::{BufferUsageFlags, CommandBuffer, PipelineStageFlags};
use gpu_allocator::MemoryLocation;
use crate::ids::{FrameIndex, SliceIndex};
use crate::render::device::device_context::DeviceContext;
use crate::render::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::factories::buffer::frame_buffer::frame_buffer::FrameBuffer;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use crate::render::statistics::interval::interval_measurement::{IntervalMeasurement, IntervalMeasurementResult};

pub struct GpuIntervalMeasurement {
    query_pool: ManagedQueryPool,
    buffer: FrameBuffer<SliceBuffer<IntervalMeasurementResult>>,

    frame_capacity: u32,
    timestamp_period: f64,
}

impl GpuIntervalMeasurement {
    pub fn new(
        device_context: &DeviceContext,
        label: &str,
        query_pool_factory: &QueryPoolFactory,
        buffer_factory: &ManagedBufferFactory,
        frame_capacity: u32,
        frame_count: u32,
    ) -> Result<Self> {
        let label = &format!("{}_interval_measurement", label);
        let total_capacity = frame_count * frame_capacity * IntervalMeasurement::Count as u32;
        
        let query_pool = query_pool_factory
            .create_query_pool(total_capacity, label)?;
        let buffer = BufferBuilder::slice::<IntervalMeasurementResult>(total_capacity)
            .per_frame(frame_count)
            .build(
                &buffer_factory,
                label,
                BufferUsageFlags::SHADER_DEVICE_ADDRESS
                    | BufferUsageFlags::STORAGE_BUFFER
                    | BufferUsageFlags::TRANSFER_DST,
                MemoryLocation::GpuToCpu,
            )?;

        Ok(Self {
            query_pool,
            buffer,
            
            frame_capacity,
            
            timestamp_period: device_context.physical_device_info.timestamp_period as f64,
        })
    }

    pub fn reset(
        &self,
        command_buffer: CommandBuffer,
        frame_index: FrameIndex,
    ) {
        self.query_pool.reset(
            command_buffer,
            frame_index.value * self.frame_capacity * IntervalMeasurement::Count as u32,
            self.frame_capacity * IntervalMeasurement::Count as u32,
        );
    }

    pub fn record(
        &self,
        command_buffer: CommandBuffer,
        frame_index: FrameIndex,
        index: u32,
        interval_measurement: IntervalMeasurement,
    ) {
        let stage = match interval_measurement {
            IntervalMeasurement::Start => PipelineStageFlags::TOP_OF_PIPE,
            IntervalMeasurement::End => PipelineStageFlags::BOTTOM_OF_PIPE,
            IntervalMeasurement::Count => unreachable!()
        };
        let query = (frame_index.value * self.frame_capacity) * IntervalMeasurement::Count as u32
            + index * IntervalMeasurement::Count as u32
            + interval_measurement as u32;

        self.query_pool.record(command_buffer, stage, query)
    }

    pub fn extract(&self, command_buffer: CommandBuffer, frame_index: FrameIndex) {
        let buffer_view = self.buffer.frame(frame_index).slice_at(SliceIndex::ZERO);

        self.query_pool
            .copy_to_buffer::<u64>(
                command_buffer,
                frame_index.value * self.frame_capacity * IntervalMeasurement::Count as u32,
                self.frame_capacity * IntervalMeasurement::Count as u32,
                &buffer_view,
            );
    }

    pub fn collect(&self, frame_index: FrameIndex) -> Vec<u64> {
        let buffer_view = self.buffer.frame(frame_index).slice_at(SliceIndex::ZERO);

        let mapped_ptr = buffer_view.mapped_ptr() as *const IntervalMeasurementResult;

        let results_count = self.frame_capacity as usize;

        let raw_slice = unsafe { from_raw_parts(mapped_ptr, results_count) };
        
        raw_slice.iter()
            .map(|raw| {
                ((raw.end - raw.start) as f64 * self.timestamp_period) as u64
            })
            .collect::<Vec<_>>()
    }

    pub fn destroy(self, buffer_factory: &ManagedBufferFactory) -> Result<()> {
        self.query_pool.destroy();
        buffer_factory.destroy_buffer(self.buffer.into_managed_buffer())?;

        Ok(())
    }
}
