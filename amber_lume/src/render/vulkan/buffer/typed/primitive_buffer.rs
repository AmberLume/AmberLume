use crate::render::vulkan::buffer::buffer::Buffer;
use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use gpu_allocator::MemoryLocation;

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct PrimitiveGpuData {
    pub index_offset: u32,
    pub index_count: u32,
    pub vertex_offset: u32,
    _pad0: u32,
}

impl PrimitiveGpuData {
    pub fn create(
        index_count: u32,
        index_offset: u32,
        vertex_offset: u32,
    ) -> Self {
        Self {
            index_offset,
            index_count,
            vertex_offset,
            _pad0: 0,
        }
    }
}

pub fn create_primitive_buffer(
    device_context: &mut DeviceContext,
    capacity: usize,
) -> Result<Buffer> {
    let size_of = size_of::<PrimitiveGpuData>() as DeviceSize;

    Buffer::create(
        device_context,
        "primitive_buffer",
        size_of * capacity as DeviceSize,
        size_of,
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
    )
}
