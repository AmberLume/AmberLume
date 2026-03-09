use anyhow::Result;
use ash::vk;
use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3, Vec4};
use gpu_allocator::MemoryLocation;
use vk::BufferUsageFlags;
use builder::data::submesh_data::SubmeshData;
use crate::render::vulkan::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::vulkan::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct VertexGpuData {
    pub position: [f32; 3],
    pub _pad0: f32,
    pub normal: [f32; 3],
    pub _pad1: f32,
    pub tangent: [f32; 4],
    pub uv: [f32; 2],
    pub _pad2: [f32; 2],
}

impl VertexGpuData {
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

    pub fn from(submesh_data: &SubmeshData, index: usize) -> Self {
        let position = &submesh_data.positions[index];
        let normal = &submesh_data.normals[index];
        let tangent = &submesh_data.tangents[index];
        let uv = &submesh_data.uvs[index];

        Self::create(
            Vec3::new(position[0], position[1], position[2]),
            Vec3::new(normal[0], normal[1], normal[2]),
            Vec4::new(tangent[0], tangent[1], tangent[2], tangent[3]),
            Vec2::new(uv[0], uv[1]),
        )
    }
}
pub fn create_vertex_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: u32,
) -> Result<SliceBuffer<VertexGpuData>> {
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
