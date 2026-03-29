use anyhow::Result;
use ash::vk;
use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3, Vec4};
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
    pub fn create(position: Vec3, normal: Vec3, tangent: Vec4, uv: Vec2) -> Self {
        Self {
            position: [position.x, position.y, position.z],
            _pad0: 0.0,
            normal: [normal.x, normal.y, normal.z],
            _pad1: 0.0,
            tangent: [tangent.x, tangent.y, tangent.z, tangent.w],
            uv: [uv.x, uv.y],
            _pad2: [0.0; 2],
        }
    }

    pub fn from(submesh_data: &ArchivedSubmeshData, index: usize) -> Self {
        let position = &submesh_data.positions[index];
        let normal = &submesh_data.normals[index];
        let tangent = &submesh_data.tangents[index];
        let uv = &submesh_data.uvs[index];

        Self::create(
            Vec3::new(position[0].into(), position[1].into(), position[2].into()),
            Vec3::new(normal[0].into(), normal[1].into(), normal[2].into()),
            Vec4::new(tangent[0].into(), tangent[1].into(), tangent[2].into(), tangent[3].into()),
            Vec2::new(uv[0].into(), uv[1].into()),
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
