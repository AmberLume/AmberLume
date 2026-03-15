use crate::render::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use anyhow::Result;
use ash::vk::BufferUsageFlags;
use bytemuck::{Pod, Zeroable};
use gpu_allocator::MemoryLocation;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct ModelGpuData {
    pub submesh_offset: u32,
    pub submesh_count: u32,
    _pad0: [u32; 2],
}

impl ModelGpuData {
    pub fn create(submesh_offset: u32, submesh_count: u32) -> Self {
        Self {
            submesh_offset,
            submesh_count,
            _pad0: [0; 2],
        }
    }
}

pub fn create_model_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: u32,
) -> Result<SliceBuffer<ModelGpuData>> {
    BufferBuilder::slice(capacity).build(
        buffer_factory,
        "model_buffer",
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
    )
}
