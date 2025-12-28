use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use bytemuck::{Pod, Zeroable};
use gpu_allocator::MemoryLocation;
use crate::render::vulkan::buffer::buffer::Buffer;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct ModelGpuData {
    pub primitive_offset: u32,
    pub primitive_count: u32,
    _pad0: [u32; 2],
}

impl ModelGpuData {
    pub fn create(
        primitive_offset: u32,
        primitive_count: u32,
    ) -> Self {
        Self {
            primitive_offset,
            primitive_count,
            _pad0: [0; 2],
        }
    }
}

pub fn create_model_buffer(
    device_context: &mut DeviceContext,
    capacity: usize,
) -> Result<Buffer> {
    let size_of = size_of::<ModelGpuData>() as DeviceSize;

    Buffer::create(
        device_context,
        "model_buffer",
        size_of * capacity as DeviceSize,
        size_of,
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::CpuToGpu,
    )
}
