use crate::render::vulkan::buffer::index_buffer::IndexBuffer;
use crate::render::vulkan::buffer::vertex_buffer::VertexBuffer;
use anyhow::Result;
use ash::Device;
use gpu_allocator::vulkan::Allocator;

pub struct BufferManager {
    pub vertex_buffer: VertexBuffer,
    pub index_buffer: IndexBuffer,
}

impl BufferManager {
    pub fn create(device: Device, allocator: &mut Allocator) -> Result<BufferManager> {
        let vertex_buffer = VertexBuffer::create(device.clone(), allocator, 100_000)?;

        let index_buffer = IndexBuffer::create(device.clone(), allocator, 100_000)?;

        Ok(BufferManager {
            vertex_buffer,
            index_buffer,
        })
    }
}

impl Drop for BufferManager {
    fn drop(&mut self) {}
}
