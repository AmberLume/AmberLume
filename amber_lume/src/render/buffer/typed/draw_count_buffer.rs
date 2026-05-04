use anyhow::Result;
use ash::vk::BufferUsageFlags;
use gpu_allocator::MemoryLocation;
use crate::render::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::factories::buffer::chunk_buffer::chunk_buffer::ChunkBuffer;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::buffer::typed_buffer::typed_buffer::TypedBuffer;

pub fn create_draw_count_buffer(
    buffer_factory: &ManagedBufferFactory,
    chunk_count: u32,
) -> Result<ChunkBuffer<TypedBuffer<u32>>> {
    BufferBuilder::typed()
        .chunked(chunk_count)
        .build(
            buffer_factory,
            "draw_count_buffer",
            BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::INDIRECT_BUFFER,
            MemoryLocation::GpuOnly,
        )
}
