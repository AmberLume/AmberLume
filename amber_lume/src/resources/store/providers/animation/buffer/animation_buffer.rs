use gpu::BufferBuilder;
use gpu::ManagedBufferFactory;
use gpu::SliceBuffer;
use anyhow::Result;
use ash::vk::BufferUsageFlags;
use bytemuck::{Pod, Zeroable};
use gpu_allocator::MemoryLocation;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct AnimationGPU {
    pub offset: u32,
    pub bone_count: u32,
    pub frame_count: u32,
    pub duration: f32,
    pub fps: f32,

    _pad0: [u32; 3],
}

impl AnimationGPU {
    pub fn create(offset: u32, bone_count: u32, frame_count: u32, duration: f32, fps: f32) -> Self {
        Self {
            offset,
            bone_count,
            frame_count,
            duration,
            fps,

            _pad0: [0; 3],
        }
    }
}

pub fn create_animation_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: u32,
) -> Result<SliceBuffer<AnimationGPU>> {
    BufferBuilder::slice(capacity).build(
        buffer_factory,
        "animation",
        BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
    )
}
