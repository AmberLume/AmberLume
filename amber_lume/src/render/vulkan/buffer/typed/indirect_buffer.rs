use anyhow::{Result};
use ash::vk::{BufferUsageFlags, DeviceSize};
use gpu_allocator::MemoryLocation;
use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::vulkan::factories::buffer::pool_buffer::PoolBuffer;

#[repr(C)]
pub struct IndirectGpuData {
    pub index_count: u32,
    pub instance_count: u32,
    pub index_offset: u32,
    pub vertex_offset: i32,
    pub instance_offset: u32,
}

pub fn create_indirect_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: usize,
) -> Result<PoolBuffer> {
    let item_size = size_of::<IndirectGpuData>() as DeviceSize;
    
    let managed_buffer = buffer_factory.create_managed_buffer(
        "indirect_buffer",
        item_size * capacity as DeviceSize,
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::INDIRECT_BUFFER,
        MemoryLocation::GpuOnly,
    )?;
    
    Ok(PoolBuffer::handle(managed_buffer, item_size))
}
