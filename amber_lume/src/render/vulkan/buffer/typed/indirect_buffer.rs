use crate::render::vulkan::device_context::DeviceContext;
use anyhow::{Result};
use ash::vk::{BufferUsageFlags, DeviceSize};
use gpu_allocator::MemoryLocation;
use crate::render::vulkan::buffer::buffer::Buffer;

#[repr(C)]
pub struct IndirectGpuData {
    pub index_count: u32,
    pub instance_count: u32,
    pub index_offset: u32,
    pub vertex_offset: i32,
    pub instance_offset: u32,
}

pub fn create_indirect_buffer(
    device_context: &mut DeviceContext,
    capacity: usize,
) -> Result<Buffer> {
    Buffer::create(
        device_context,
        "indirect_buffer",
        capacity,
        size_of::<IndirectGpuData>() as DeviceSize,
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::INDIRECT_BUFFER,
        MemoryLocation::GpuOnly,
    )
}
