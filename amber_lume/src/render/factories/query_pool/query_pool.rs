use crate::render::factories::buffer::managed_buffer::ManagedBuffer;
use crate::render::factories::buffer::view::buffer_view::BufferView;
use crate::render::utils::debug_utils::DebugUtils;
use anyhow::Result;
use ash::Device;
use ash::vk::{
    CommandBuffer, DeviceSize, PipelineStageFlags, QueryPool, QueryPoolCreateInfo,
    QueryResultFlags, QueryType,
};
use tracing::info;

pub struct ManagedQueryPool {
    device: Device,

    handle: QueryPool,

    label: String,
}

impl ManagedQueryPool {
    pub fn new(
        device: Device,
        capacity: u32,
        debug_utils: &DebugUtils,
        label: &str,
    ) -> Result<Self> {
        let query_pool_info = QueryPoolCreateInfo::default()
            .query_type(QueryType::TIMESTAMP)
            .query_count(capacity);

        let handle = unsafe { device.create_query_pool(&query_pool_info, None)? };

        unsafe { device.reset_query_pool(handle, 0, capacity) };

        debug_utils.label(handle, &format!("query_pool_{}", label));

        Ok(Self {
            device,
            handle,
            label: label.to_string(),
        })
    }

    pub fn reset(&self, command_buffer: CommandBuffer, start: u32, count: u32) {
        unsafe { self.device.cmd_reset_query_pool(command_buffer, self.handle, start, count); }
    }

    pub fn record(&self, command_buffer: CommandBuffer, stage: PipelineStageFlags, query: u32) {
        unsafe { self.device.cmd_write_timestamp(command_buffer, stage, self.handle, query); }
    }

    pub fn copy_to_buffer<T>(
        &self,
        command_buffer: CommandBuffer,
        start: u32,
        count: u32,
        buffer_view: &BufferView<ManagedBuffer>,
    ) {
        unsafe {
            self.device.cmd_copy_query_pool_results(
                command_buffer,
                self.handle,
                start,
                count,
                buffer_view.handle(),
                buffer_view.offset(),
                size_of::<T>() as DeviceSize,
                QueryResultFlags::TYPE_64 | QueryResultFlags::WAIT,
            );
        }
    }

    pub fn destroy(self) {
        unsafe { self.device.destroy_query_pool(self.handle, None) }

        info!("QueryPool {} destroyed", self.label);
    }
}
