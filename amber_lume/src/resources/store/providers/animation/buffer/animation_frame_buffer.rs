use gpu::BufferBuilder;
use gpu::ManagedBufferFactory;
use gpu::SliceBuffer;
use anyhow::Result;
use ash::vk::BufferUsageFlags;
use bytemuck::{Pod, Zeroable};
use gpu_allocator::MemoryLocation;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct AnimationFrameGPU {
    pub translation: [f32; 3],
    _pad0: u32,
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
    _pad1: u32,
}

impl AnimationFrameGPU {
    pub fn create(
        translation: [f32; 3],
        rotation: [f32; 4],
        scale: [f32; 3],
    ) -> Self {
        Self {
            translation,
            _pad0: 0,
            rotation,
            scale,
            _pad1: 0,
        }
    }
}

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
