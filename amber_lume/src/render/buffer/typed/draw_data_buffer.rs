use anyhow::Result;
use ash::vk::BufferUsageFlags;
use bytemuck::{Pod, Zeroable};
use gpu_allocator::MemoryLocation;
use crate::render::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::factories::buffer::chunk_buffer::chunk_buffer::ChunkBuffer;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct DrawDataGPU {
    pub entity_index: u32,
    pub submesh_index: u32,
    pub cascade_mask: u32,
    _pad0: u32,
}

pub fn create_draw_data_buffer(
    buffer_factory: &ManagedBufferFactory,
    chunk_count: u32,
    capacity: u32,
) -> Result<ChunkBuffer<SliceBuffer<DrawDataGPU>>> {
    BufferBuilder::slice(capacity)
        .chunked(chunk_count)
        .build(
            buffer_factory,
            "draw_data_buffer",
            BufferUsageFlags::STORAGE_BUFFER,
            MemoryLocation::GpuOnly,
        )
}
