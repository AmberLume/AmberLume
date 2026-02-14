use crate::render::vulkan::device_context::DeviceContext;
use ash::vk::{BufferUsageFlags, CommandBuffer, PipelineStageFlags, QueryPool, QueryPoolCreateInfo, QueryResultFlags, QueryType};
use anyhow::Result;
use ash::Device;
use gpu_allocator::MemoryLocation;
use crate::render::vulkan::factories::buffer::managed_buffer::ManagedBuffer;
use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;

pub struct RawFrameStat {
    query_pool: QueryPool,

    managed_buffer: ManagedBuffer,
    mapped_ptr: *mut u64,
}

impl RawFrameStat {
    pub fn new(
        device: &Device,
        buffer_factory: &ManagedBufferFactory,
    ) -> Result<Self> {
        let query_pool_info = QueryPoolCreateInfo::default()
            .query_type(QueryType::TIMESTAMP)
            .query_count(2);

        let query_pool = unsafe { device.create_query_pool(&query_pool_info, None)? };

        unsafe { device.reset_query_pool(query_pool, 0, 2) };

        let managed_buffer = buffer_factory.create_managed_buffer(
            "raw_frame_stat",
            16,
            BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::CpuToGpu,
        )?;
        
        let mapped_ptr = managed_buffer.allocation.mapped_ptr()
            .unwrap()
            .as_ptr() as *mut u64;

        Ok(Self {
            query_pool,

            managed_buffer,
            mapped_ptr,
        })
    }

    fn reset(&self, device_context: &DeviceContext, command_buffer: CommandBuffer) {
        unsafe {
            device_context.device.cmd_reset_query_pool(command_buffer, self.query_pool, 0, 2)
        };
    }

    pub fn start(&self, device_context: &DeviceContext, command_buffer: CommandBuffer) {
        self.reset(&device_context, command_buffer);

        unsafe {
            device_context.device.cmd_write_timestamp(
                command_buffer,
                PipelineStageFlags::TOP_OF_PIPE,
                self.query_pool,
                0,
            )
        }
    }

    pub fn finish(&self, device_context: &DeviceContext, command_buffer: CommandBuffer) {
        unsafe {
            device_context.device.cmd_write_timestamp(
                command_buffer,
                PipelineStageFlags::BOTTOM_OF_PIPE,
                self.query_pool,
                1,
            )
        }

        unsafe {
            device_context.device.cmd_copy_query_pool_results(
                command_buffer,
                self.query_pool,
                0, 
                2,
                self.managed_buffer.handle,
                0, 
                8,
                QueryResultFlags::TYPE_64 | QueryResultFlags::WAIT,
            );
        }
    }

    pub fn pull(&self) -> [u64; 2] {
        unsafe {
            let start = *self.mapped_ptr;
            let end = *self.mapped_ptr.add(1);

            [start, end]
        }
    }

    pub fn destroy(
        self, 
        device: &Device,
        buffer_factory: &ManagedBufferFactory,
    ) {
        unsafe { device.destroy_query_pool(self.query_pool, None) }
        
        buffer_factory.destroy(self.managed_buffer);
    }
}
