use anyhow::Result;
use ash::vk;
use gpu_allocator::MemoryLocation;
use vk::BufferUsageFlags;
use crate::render::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;

pub fn create_index_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: u32,
) -> Result<SliceBuffer<u32>> {
    BufferBuilder::slice(capacity)
        .build(
            buffer_factory,
            "index",
            BufferUsageFlags::INDEX_BUFFER
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuOnly,
        )
}
