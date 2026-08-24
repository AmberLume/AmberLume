use gpu::ManagedBufferFactory;
use gpu::ManagedBuffer;
use gpu_data::AnimationGPU;
use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use gpu_allocator::MemoryLocation;

pub fn create_animation_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: u32,
) -> Result<ManagedBuffer> {
    buffer_factory.create_managed_buffer(
        "animation",
        capacity as DeviceSize * size_of::<AnimationGPU>() as DeviceSize,
        BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
    )
}
