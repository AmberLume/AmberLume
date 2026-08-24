use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use gpu::ManagedBuffer;
use gpu::ManagedBufferFactory;
use gpu_allocator::MemoryLocation;
use gpu_data::SubmeshGPU;

pub fn create_submesh_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: u32,
) -> Result<ManagedBuffer> {
    buffer_factory.create_managed_buffer(
        "submesh",
        capacity as DeviceSize * size_of::<SubmeshGPU>() as DeviceSize,
        BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
    )
}
