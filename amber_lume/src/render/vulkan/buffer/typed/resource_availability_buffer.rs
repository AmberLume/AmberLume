use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use gpu_allocator::MemoryLocation;
use crate::render::vulkan::buffer::buffer::Buffer;

pub fn create_resource_availability_buffer(
    device_context: &mut DeviceContext,
    tag: &str,
    capacity: usize,
) -> Result<Buffer> {
    let size_of = size_of::<u32>();

    Buffer::create(
        device_context,
        &format!("{}_availability_buffer", tag),
        (size_of * capacity) as DeviceSize,
        size_of as DeviceSize,
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::CpuToGpu,
    )
}
