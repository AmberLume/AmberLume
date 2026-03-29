use anyhow::Result;
use ash::vk;
use bytemuck::{Pod, Zeroable};
use gpu_allocator::MemoryLocation;
use vk::BufferUsageFlags;
use builder::data::submesh_data::ArchivedSubmeshData;
use crate::render::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct VertexGPU {
    pub position: [f32; 3],
    pub _pad0: f32,
    pub normal: [f32; 3],
    pub _pad1: f32,
    pub tangent: [f32; 4],
    pub uv: [f32; 2],
    pub _pad2: [f32; 2],
}

impl VertexGPU {
    pub fn new(position: [f32; 3], normal: [f32; 3], tangent: [f32; 4], uv: [f32; 2]) -> Self {
        Self {
            position,
            _pad0: 0.0,
            normal,
            _pad1: 0.0,
            tangent,
            uv,
            _pad2: [0.0; 2],
        }
    }

    pub fn from(submesh_data: &ArchivedSubmeshData, index: usize) -> Self {
        let position = &submesh_data.positions[index];
        let normal = &submesh_data.normals[index];
        let tangent = &submesh_data.tangents[index];
        let uv = &submesh_data.uvs[index];

        Self::new(
            position.map(|v| v.into()),
            normal.map(|v| v.into()),
            tangent.map(|v| v.into()),
            uv.map(|v| v.into()),
        )
    }
}
pub fn create_vertex_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: u32,
) -> Result<SliceBuffer<VertexGPU>> {
    BufferBuilder::slice(capacity)
        .build(
            buffer_factory,
            "vertex_buffer",
            BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuOnly,
        )
}
