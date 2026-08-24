use anyhow::Result;
use ash::vk;
use gpu::ManagedBuffer;
use gpu::ManagedBufferFactory;
use gpu_allocator::MemoryLocation;
use vk::{BufferUsageFlags, DeviceSize};

pub fn create_index_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: u32,
    ray_tracing: bool,
) -> Result<ManagedBuffer> {
    let mut usage = BufferUsageFlags::INDEX_BUFFER | BufferUsageFlags::TRANSFER_DST;

    if ray_tracing {
        usage |= BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR;
    }

    buffer_factory.create_managed_buffer(
        "index",
        capacity as DeviceSize * size_of::<u32>() as DeviceSize,
        usage,
        MemoryLocation::GpuOnly,
    )
}
