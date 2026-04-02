use crate::render::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use anyhow::Result;
use ash::vk::BufferUsageFlags;
use bytemuck::{Pod, Zeroable};
use gpu_allocator::MemoryLocation;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct SkeletonGPU {
    pub offset: u32,
    pub count: u32,

    _pad0: [u32; 2],
}

impl SkeletonGPU {
    pub fn create(offset: u32, count: u32) -> Self {
        Self {
            offset,
            count,

            _pad0: [0; 2],
        }
    }
}

pub fn create_skeleton_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: u32,
) -> Result<SliceBuffer<SkeletonGPU>> {
    BufferBuilder::slice(capacity).build(
        buffer_factory,
        "skeleton",
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
    )
}
