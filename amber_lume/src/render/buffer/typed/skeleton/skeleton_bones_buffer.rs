use crate::render::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use anyhow::Result;
use ash::vk::BufferUsageFlags;
use bytemuck::{Pod, Zeroable};
use gpu_allocator::MemoryLocation;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct SkeletonBoneGPU {
    pub parent: i32,

    _pad0: [u32; 3],

    pub inverse_bind_matrix: [[f32; 4]; 4],
}

impl SkeletonBoneGPU {
    pub fn create(parent: i32, inverse_bind_matrix: [[f32; 4]; 4]) -> Self {
        Self {
            parent,
            
            _pad0: [0; 3],
            
            inverse_bind_matrix,
        }
    }
}

pub fn create_skeleton_bones_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: u32,
) -> Result<SliceBuffer<SkeletonBoneGPU>> {
    BufferBuilder::slice(capacity).build(
        buffer_factory,
        "skeleton_bones",
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
    )
}
