use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use bytemuck::{Pod, Zeroable};
use gpu_allocator::MemoryLocation;
use crate::render::vulkan::buffer::buffer::Buffer;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct MaterialGpuData {
    pub base_color: [f32; 4],

    pub base_color_texture_index: u32,
    _pad0: [u32; 3],
}

impl MaterialGpuData {
    pub fn create(
        base_color: [f32; 4],
        base_color_texture_index: u32,
    ) -> Self {
        Self {
            base_color,

            base_color_texture_index,
            _pad0: [0; 3],
        }
    }
}

pub fn create_material_buffer(
    device_context: &mut DeviceContext,
    capacity: usize,
) -> Result<Buffer> {
    Buffer::create(
        device_context,
        "material_buffer",
        capacity,
        size_of::<MaterialGpuData>() as DeviceSize,
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::CpuToGpu,
    )
}
