use crate::render::vulkan::buffer::buffer::Buffer;
use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;
use ash::vk;
use gpu_allocator::MemoryLocation;
use vk::{BufferUsageFlags, DeviceSize};

pub fn create_index_buffer(
    device_context: &mut DeviceContext,
    capacity: usize,
) -> Result<Buffer> {
    Buffer::create(
        device_context,
        "index_buffer",
        capacity,
        size_of::<u32>() as DeviceSize,
        BufferUsageFlags::INDEX_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
    )
}
