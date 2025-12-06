use crate::render::vulkan::buffer::index_buffer::IndexBuffer;
use crate::render::vulkan::buffer::vertex_buffer::VertexBuffer;
use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;
use gpu_allocator::vulkan::Allocator;
use tracing::info;

pub struct BufferManager {
    pub vertex_buffer: VertexBuffer,
    pub index_buffer: IndexBuffer,
}

impl BufferManager {
    pub fn create(
        device_context: &DeviceContext,
        allocator: &mut Allocator,
    ) -> Result<BufferManager> {
        let vertex_buffer = VertexBuffer::create(&device_context, allocator, 100_000)?;

        let index_buffer = IndexBuffer::create(&device_context, allocator, 100_000)?;

        Ok(BufferManager {
            vertex_buffer,
            index_buffer,
        })
    }

    pub fn destroy(&mut self) -> Result<()> {
        self.index_buffer.destroy()?;
        self.vertex_buffer.destroy()?;

        info!("BufferManager destroyed");

        Ok(())
    }
}
