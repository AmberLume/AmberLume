use crate::render::vulkan::buffer::buffer::Buffer;
use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use gpu_allocator::MemoryLocation;

pub fn create_staging_buffer(
    device_context: &mut DeviceContext,
    tag: &str,
    size: DeviceSize,
) -> Result<Buffer> {
    Buffer::create(
        device_context,
        &format!("staging_buffer_{}", tag),
        size,
        0,
        BufferUsageFlags::TRANSFER_SRC,
        MemoryLocation::CpuToGpu,
    )
}
