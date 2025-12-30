use crate::render::vulkan::buffer::buffer::Buffer;
use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;
use ash::vk;
use glam::{Vec2, Vec3};
use gpu_allocator::MemoryLocation;
use vk::{BufferUsageFlags, DeviceSize};
use alpaca::data::common::primitive_data::PrimitiveData;

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct VertexGpuData {
    pub position: [f32; 3],
    pub _pad0: f32,
    pub normal: [f32; 3],
    pub _pad1: f32,
    pub uv: [f32; 2],
    pub _pad2: [f32; 2],
}

impl VertexGpuData {
    pub fn create(position: Vec3, normal: Vec3, uv: Vec2) -> Self {
        Self {
            position: [position.x, position.y, position.z],
            _pad0: 0.0,
            normal: [normal.x, normal.y, normal.z],
            _pad1: 0.0,
            uv: [uv.x, uv.y],
            _pad2: [0.0; 2],
        }
    }

    pub fn from(primitive_data: &PrimitiveData, index: usize) -> Self {
        let position = &primitive_data.vertices[index];
        let normal = &primitive_data.normals[index];
        let uv = &primitive_data.uv[index];

        Self::create(
            Vec3::new(position[0], position[1], position[2]),
            Vec3::new(normal[0], normal[1], normal[2]),
            Vec2::new(uv[0], uv[1]),
        )
    }
}
pub fn create_vertex_buffer(
    device_context: &mut DeviceContext,
    capacity: usize,
) -> Result<Buffer> {
    let size_of = size_of::<VertexGpuData>() as DeviceSize;

    Buffer::create(
        device_context,
        "vertex_buffer",
        size_of * capacity as DeviceSize,
        size_of,
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
    )
}
