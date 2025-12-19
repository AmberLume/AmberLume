use crate::render::vulkan::buffer::buffer::Buffer;
use crate::render::vulkan::data::vertex::Vertex;
use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;
use ash::vk;
use gpu_allocator::MemoryLocation::GpuOnly;
use tracing::info;
use vk::{BufferUsageFlags, DeviceAddress, DeviceSize};

pub struct VertexBuffer {
    pub buffer: Buffer,
}

impl VertexBuffer {
    pub fn create(device_context: &mut DeviceContext, capacity: usize) -> Result<Self> {
        let size = (size_of::<Vertex>() * capacity) as DeviceSize;

        let buffer = Buffer::create(
            device_context,
            size,
            BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::TRANSFER_DST,
            GpuOnly,
            "vertex_buffer",
        )?;

        Ok(Self { buffer })
    }

    pub fn allocate_space(&self, vertex_count: usize) -> Result<u32> {
        let size_bytes = (vertex_count * size_of::<Vertex>()) as DeviceSize;
        let alignment = 16;

        let offset_bytes = self.buffer.allocate_space(size_bytes, alignment)?;

        Ok((offset_bytes / size_of::<Vertex>() as u64) as u32)
    }

    pub fn destroy(&mut self) -> Result<()> {
        self.buffer.destroy()?;

        info!("VertexBuffer destroyed");

        Ok(())
    }
}
