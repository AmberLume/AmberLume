use crate::render::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use anyhow::Result;
use ash::vk::BufferUsageFlags;
use bytemuck::{Pod, Zeroable};
use gpu_allocator::MemoryLocation;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct SubmeshGPU {
    pub index_offset: u32,
    pub index_count: u32,
    pub vertex_offset: u32,

    pub material_index: u32,

    pub bounds_min: [f32; 4],
    pub bounds_max: [f32; 4],
}

impl SubmeshGPU {
    pub fn create(
        index_count: u32,
        index_offset: u32,
        vertex_offset: u32,
        material_index: u32,
        bounds: [f32; 6],
    ) -> Self {
        Self {
            index_offset,
            index_count,
            vertex_offset,
            material_index,
            bounds_min: [bounds[0], bounds[1], bounds[2], 0.0],
            bounds_max: [bounds[3], bounds[4], bounds[5], 0.0],
        }
    }
}

pub fn create_submesh_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: u32,
) -> Result<SliceBuffer<SubmeshGPU>> {
    BufferBuilder::slice(capacity).build(
        buffer_factory,
        "submesh",
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
    )
}
