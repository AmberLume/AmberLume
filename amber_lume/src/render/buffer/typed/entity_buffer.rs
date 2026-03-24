use anyhow::Result;
use ash::vk::BufferUsageFlags;
use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use gpu_allocator::MemoryLocation;
use crate::render::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::factories::buffer::frame_buffer::frame_buffer::FrameBuffer;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct EntityGpuData {
    pub transform_matrix: [[f32; 4]; 4],
    pub mesh_index: u32,
    _pad0: [f32; 3],
}

impl EntityGpuData {
    pub fn create(transform_matrix: Mat4, mesh_index: u32) -> Self {
        Self {
            transform_matrix: transform_matrix.to_cols_array_2d(),
            mesh_index,
            _pad0: [0.0; 3],
        }
    }
}

pub fn create_entity_buffer(
    buffer_factory: &ManagedBufferFactory,
    frame_count: u32,
    capacity: u32,
) -> Result<FrameBuffer<SliceBuffer<EntityGpuData>>> {
    BufferBuilder::slice(capacity)
        .per_frame(frame_count)
        .build(
            buffer_factory, 
            "entity_buffer",
            BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::CpuToGpu,
        )
}
