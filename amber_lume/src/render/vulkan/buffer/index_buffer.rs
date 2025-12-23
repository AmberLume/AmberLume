use crate::render::vulkan::buffer::buffer::Buffer;
use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;
use ash::vk;
use gpu_allocator::MemoryLocation;
use tracing::info;
use vk::{BufferUsageFlags, DeviceSize};

pub struct IndexBuffer {
    pub buffer: Buffer,
}

impl IndexBuffer {
    pub fn create(device_context: &mut DeviceContext, capacity: usize) -> Result<Self> {
        let size = (size_of::<u32>() * capacity) as DeviceSize;

        let buffer = Buffer::create(
            device_context,
            size,
            BufferUsageFlags::INDEX_BUFFER
                | BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuOnly,
            "index_buffer",
        )?;

        Ok(Self { buffer })
    }

    pub fn allocate_space(&self, index_count: usize) -> Result<u64> {
        let size_bytes = (index_count * size_of::<u32>()) as DeviceSize;
        let alignment = 4;

        let offset_bytes = self.buffer.allocate_space(size_bytes, alignment)?;

        Ok(offset_bytes)
    }

    pub fn destroy(&mut self) -> Result<()> {
        self.buffer.destroy()?;

        info!("IndexBuffer destroyed");

        Ok(())
    }
}
