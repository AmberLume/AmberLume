use crate::render::vulkan::buffer::buffer::Buffer;
use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use glam::Mat4;
use gpu_allocator::MemoryLocation;

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct EntityGpuData {
    pub transform_matrix: Mat4,
    pub model_index: u32,
    _pad0: [f32; 3],
}

impl EntityGpuData {
    pub fn create(transform_matrix: Mat4, model_index: u32) -> Self {
        Self {
            transform_matrix,
            model_index,
            _pad0: [0.0; 3],
        }
    }
}

pub fn create_entity_buffer(
    device_context: &mut DeviceContext,
    capacity: usize,
) -> Result<Buffer> {
    let size_of = size_of::<EntityGpuData>() as DeviceSize;
  
    Buffer::create(
        device_context,
        "entity_buffer",
        size_of * capacity as DeviceSize,
        size_of,
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::CpuToGpu,
    )
}
