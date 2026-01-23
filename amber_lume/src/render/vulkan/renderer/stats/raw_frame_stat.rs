use crate::render::vulkan::device_context::DeviceContext;
use ash::vk::{Buffer, BufferCreateInfo, BufferUsageFlags, CommandBuffer, PipelineStageFlags, QueryPool, QueryPoolCreateInfo, QueryResultFlags, QueryType};
use anyhow::Result;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{AllocationCreateDesc, AllocationScheme};

pub struct RawFrameStat {
    query_pool: QueryPool,

    result_buffer: Buffer,
    mapped_ptr: *mut u64,
}

impl RawFrameStat {
    pub fn new(device_context: &DeviceContext) -> Result<Self> {
        let device = &device_context.device;

        let query_pool_info = QueryPoolCreateInfo::default()
            .query_type(QueryType::TIMESTAMP)
            .query_count(2);

        let query_pool = unsafe { device.create_query_pool(&query_pool_info, None)? };

        unsafe { device.reset_query_pool(query_pool, 0, 2) };

        let buffer_info = BufferCreateInfo::default()
            .size(16)
            .usage(BufferUsageFlags::TRANSFER_DST);
        let result_buffer = unsafe { device.create_buffer(&buffer_info, None)? };

        let requirements = unsafe { device.get_buffer_memory_requirements(result_buffer) };
        let allocation = {
            let mut allocator = device_context.allocator.lock().unwrap();

            allocator.allocate(&AllocationCreateDesc {
                name: "frame_stats_buffer",
                requirements,
                location: MemoryLocation::CpuToGpu,
                linear: true,
                allocation_scheme: AllocationScheme::GpuAllocatorManaged,
            })?
        };

        unsafe { device.bind_buffer_memory(result_buffer, allocation.memory(), allocation.offset())? };

        let mapped_ptr = allocation.mapped_ptr()
            .unwrap()
            .as_ptr() as *mut u64;

        Ok(Self {
            query_pool,

            result_buffer,
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
                0, 2,
                self.result_buffer,
                0, 8,
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

    pub fn destroy(&self, device_context: &DeviceContext) {
        unsafe { device_context.device.destroy_query_pool(self.query_pool, None) }
        unsafe { device_context.device.destroy_buffer(self.result_buffer, None) }
    }
}
