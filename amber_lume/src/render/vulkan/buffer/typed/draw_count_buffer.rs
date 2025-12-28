use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use gpu_allocator::MemoryLocation;
use crate::render::vulkan::buffer::buffer::Buffer;

pub fn create_draw_count_buffer(
    device_context: &mut DeviceContext,
) -> Result<Buffer> {
    let size_of = size_of::<u32>() as DeviceSize;

    Buffer::create(
        device_context,
        "draw_count_buffer",
        size_of,
        0,
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::TRANSFER_DST
            | BufferUsageFlags::INDIRECT_BUFFER,
        MemoryLocation::GpuOnly,
    )
}
