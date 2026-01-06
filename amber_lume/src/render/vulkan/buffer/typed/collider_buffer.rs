use ash::vk::{BufferUsageFlags, DeviceSize};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec4};
use gpu_allocator::MemoryLocation;
use crate::render::vulkan::buffer::buffer::Buffer;
use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct ColliderGpuData {
    pub transform_matrix: [[f32; 4]; 4],
    pub half_extents: [f32; 4],
    pub color: [f32; 4],
    pub shape_type: u32,
    _pad0: [u32; 3],
}

impl ColliderGpuData {
    pub fn create(
        transform_matrix: Mat4,
        half_extents: Vec4,
        color: Vec4,
        shape_type: u32,
    ) -> Self {
        Self {
            transform_matrix: transform_matrix.to_cols_array_2d(),
            half_extents: half_extents.to_array(),
            color: color.to_array(),
            shape_type,
            _pad0:[0; 3],
        }
    }
}

pub fn create_collider_buffer(
    device_context: &mut DeviceContext,
    capacity: usize,
) -> Result<Buffer> {
    Buffer::create(
        device_context,
        "collider_buffer",
        capacity,
        size_of::<ColliderGpuData>() as DeviceSize,
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::CpuToGpu,
    )
}