use crate::render::vulkan::buffer::buffer::Buffer;
use crate::render::vulkan::data::vertex::Vertex;
use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;
use ash::vk;
use ash::vk::DeviceAddress;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::Allocator;
use tracing::info;
use vk::{BufferUsageFlags, DeviceSize};

pub struct IndexBuffer {
    pub buffer: Buffer,
}

impl IndexBuffer {
    pub fn create(
        device_context: &DeviceContext,
        allocator: &mut Allocator,
        capacity: usize,
    ) -> Result<Self> {
        let size = (size_of::<u32>() * capacity) as DeviceSize;

        let buffer = Buffer::create(
            device_context,
            allocator,
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

    pub fn allocate_space(&self, index_count: usize) -> Result<DeviceSize> {
        let size = (index_count * size_of::<u32>()) as DeviceSize;
        let alignment = 4;

        self.buffer.allocate_space(size, alignment)
    }

    pub fn device_address(&self) -> Option<DeviceAddress> {
        self.buffer.device_address
    }

    pub fn destroy(&mut self) -> Result<()> {
        self.buffer.destroy()?;

        info!("IndexBuffer destroyed");

        Ok(())
    }
}
