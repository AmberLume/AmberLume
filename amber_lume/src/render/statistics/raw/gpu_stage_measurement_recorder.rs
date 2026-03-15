use crate::render::vulkan::factories::buffer::managed_buffer::ManagedBuffer;
use anyhow::Result;
use ash::Device;
use ash::vk::{CommandBuffer, DeviceSize, PipelineStageFlags, QueryPool, QueryPoolCreateInfo, QueryResultFlags, QueryType};
use crate::ids::FrameIndex;
use crate::render::vulkan::factories::buffer::view::buffer_view::BufferView;

pub struct GpuStageMeasurementRecorder {
    device: Device,

    query_pool: QueryPool,
}

#[repr(u32)]
pub enum GpuMeasurementStages {
    PipelineStart = 0,
    PipelineEnd = 1,

    Count = 2,
}

impl GpuStageMeasurementRecorder {
    pub fn new(
        device: Device,
        frames_in_flight: u32,
    ) -> Result<Self> {
        let query_pool_size = GpuMeasurementStages::Count as u32 * frames_in_flight;

        let query_pool_info = QueryPoolCreateInfo::default()
            .query_type(QueryType::TIMESTAMP)
            .query_count(query_pool_size);

        let query_pool = unsafe { device.create_query_pool(&query_pool_info, None)? };

        unsafe { device.reset_query_pool(query_pool, 0, query_pool_size) };

        Ok(Self {
            device,
            query_pool,
        })
    }

    pub fn reset(&self, command_buffer: CommandBuffer, frame_index: FrameIndex) {
        let start = GpuMeasurementStages::Count as u32 * frame_index.value;

        unsafe {
            self.device.cmd_reset_query_pool(
                command_buffer,
                self.query_pool,
                start,
                GpuMeasurementStages::Count as u32,
            );
        }
    }

    pub fn record(
        &self,
        command_buffer: CommandBuffer,
        stage: PipelineStageFlags,
        frame_index: FrameIndex,
        record_stage: GpuMeasurementStages,
    ) {
        let index = GpuMeasurementStages::Count as u32 * frame_index.value + record_stage as u32;

        unsafe {
            self.device.cmd_write_timestamp(
                command_buffer,
                stage,
                self.query_pool,
                index,
            );
        }
    }

    pub fn copy_to_buffer(
        &self,
        command_buffer: CommandBuffer,
        frame_index: FrameIndex,
        buffer_view: &BufferView<ManagedBuffer>,
    ) {
        let start = GpuMeasurementStages::Count as u32 * frame_index.value;

        unsafe {
            self.device.cmd_copy_query_pool_results(
                command_buffer,
                self.query_pool,
                start,
                GpuMeasurementStages::Count as u32,
                buffer_view.handle(),
                buffer_view.offset(),
                size_of::<u64>() as DeviceSize,
                QueryResultFlags::TYPE_64 | QueryResultFlags::WAIT,
            );
        }
    }

    pub fn destroy(&self) {
        unsafe { self.device.destroy_query_pool(self.query_pool, None) }
    }
}
