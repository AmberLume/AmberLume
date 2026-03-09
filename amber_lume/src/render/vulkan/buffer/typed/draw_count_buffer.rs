use anyhow::Result;
use ash::vk::BufferUsageFlags;
use gpu_allocator::MemoryLocation;
use crate::render::vulkan::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::vulkan::factories::buffer::chunk_buffer::chunk_buffer::ChunkBuffer;
use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::vulkan::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;

pub fn create_draw_count_buffer(
    buffer_factory: &ManagedBufferFactory,
    chunk_count: u32,
) -> Result<ChunkBuffer<SliceBuffer<u32>>> {
    BufferBuilder::slice(1)
        .chunked(chunk_count)
        .build(
            buffer_factory,
            "draw_count_buffer",
            BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::TRANSFER_DST
                | BufferUsageFlags::INDIRECT_BUFFER,
            MemoryLocation::GpuOnly,
        )
}
