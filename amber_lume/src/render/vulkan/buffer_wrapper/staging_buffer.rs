use crate::render::vulkan::buffer::buffer::Buffer;
use crate::render::vulkan::buffer_wrapper::buffer_wrapper::BufferWrapper;
use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use gpu_allocator::MemoryLocation;
use tracing::info;

pub struct StagingBuffer {
    pub name: String,

    pub buffer: Buffer,
}

impl StagingBuffer {
    pub fn create(device_context: &mut DeviceContext, tag: &str, size: DeviceSize) -> Result<Self> {
        let name = format!("staging_buffer_{}", tag);

        let buffer = Buffer::create(
            device_context,
            size,
            BufferUsageFlags::TRANSFER_SRC,
            MemoryLocation::CpuToGpu,
            &name,
        )?;

        Ok(Self { name, buffer })
    }
}

impl BufferWrapper for StagingBuffer {
    fn allocate_space(&self, count: usize) -> Result<u64> {
        Ok(self.buffer.allocate_space(count as DeviceSize)?)
    }

    fn destroy(&mut self) -> Result<()> {
        self.buffer.destroy()?;

        info!("Buffer '{}' destroyed", self.name);

        Ok(())
    }
}
