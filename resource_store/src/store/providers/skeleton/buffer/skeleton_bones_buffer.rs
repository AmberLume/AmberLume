use gpu::BufferBuilder;
use gpu::ManagedBufferFactory;
use gpu::SliceBuffer;
use gpu_data::SkeletonBoneGPU;
use anyhow::Result;
use ash::vk::BufferUsageFlags;
use gpu_allocator::MemoryLocation;

pub fn create_skeleton_bone_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: u32,
) -> Result<SliceBuffer<SkeletonBoneGPU>> {
    BufferBuilder::slice(capacity).build(
        buffer_factory,
        "skeleton_bone",
        BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
    )
}
