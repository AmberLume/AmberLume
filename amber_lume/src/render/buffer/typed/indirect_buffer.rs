
use crate::render::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::factories::buffer::chunk_buffer::chunk_buffer::ChunkBuffer;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use anyhow::Result;
use ash::vk::BufferUsageFlags;
use gpu_allocator::MemoryLocation;

#[repr(C)]
pub struct IndirectGPU {
    pub index_count: u32,
    pub instance_count: u32,
    pub index_offset: u32,
    pub vertex_offset: i32,
    pub instance_offset: u32,
}

pub fn create_indirect_buffer(
    buffer_factory: &ManagedBufferFactory,
    chunk_count: u32,
    capacity: u32,
) -> Result<ChunkBuffer<SliceBuffer<IndirectGPU>>> {
    BufferBuilder::slice(capacity)
        .chunked(chunk_count)
        .build(
            buffer_factory,
            "indirect_buffer",
            BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::INDIRECT_BUFFER,
            MemoryLocation::GpuOnly,
        )
}
