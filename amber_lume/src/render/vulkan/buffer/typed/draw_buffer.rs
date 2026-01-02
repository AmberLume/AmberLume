use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use bytemuck::{Pod, Zeroable};
use gpu_allocator::MemoryLocation;
use crate::render::vulkan::buffer::buffer::Buffer;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct DrawGpuData {
    pub entity_index: u32,
    pub primitive_index: u32,
    _pad: [u32; 2],
}

pub fn create_draw_buffer(
    device_context: &mut DeviceContext,
    capacity: usize,
) -> Result<Buffer> {
    Buffer::create(
        device_context,
        "draw_buffer",
        capacity,
        size_of::<DrawGpuData>() as DeviceSize,
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        MemoryLocation::GpuOnly,
    )
}
