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
    ray_tracing: bool,
) -> Result<SliceBuffer<u32>> {
    let mut usage = BufferUsageFlags::INDEX_BUFFER | BufferUsageFlags::TRANSFER_DST;

    if ray_tracing {
        usage |= BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR;
    }

    BufferBuilder::slice(capacity)
        .build(
            buffer_factory,
            "index",
            usage,
            MemoryLocation::GpuOnly,
        )
}
