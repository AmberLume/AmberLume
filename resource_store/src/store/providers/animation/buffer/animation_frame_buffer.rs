use gpu::BufferBuilder;
use gpu::ManagedBufferFactory;
use gpu::SliceBuffer;
use gpu_data::AnimationFrameGPU;
use anyhow::Result;
use ash::vk::BufferUsageFlags;
use gpu_allocator::MemoryLocation;

pub fn create_animation_frame_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: u32,
) -> Result<SliceBuffer<AnimationFrameGPU>> {
    BufferBuilder::slice(capacity).build(
        buffer_factory,
        "animation_frame",
        BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
    )
}
