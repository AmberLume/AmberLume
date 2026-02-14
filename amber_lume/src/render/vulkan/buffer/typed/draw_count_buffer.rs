use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use bytemuck::{Pod, Zeroable};
use gpu_allocator::MemoryLocation;
use crate::render::vulkan::factories::buffer::linear_buffer::LinearBuffer;
use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct DrawCountGpuData {
    pub entity_count: u32,
    pub collider_count: u32,
    _pad: [u32; 2],
}

pub fn create_draw_count_buffer(
    buffer_factory: &ManagedBufferFactory,
) -> Result<LinearBuffer> {
    let managed = buffer_factory.create_managed_buffer(
        "draw_count_buffer",
        size_of::<DrawCountGpuData>() as DeviceSize,
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::TRANSFER_DST
            | BufferUsageFlags::INDIRECT_BUFFER,
        MemoryLocation::GpuOnly,
    )?;

    Ok(LinearBuffer::handle(managed))
}
