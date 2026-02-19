use crate::render::vulkan::factories::buffer::pool_buffer::PoolBuffer;
use anyhow::Result;
use ash::vk;
use gpu_allocator::MemoryLocation;
use vk::{BufferUsageFlags, DeviceSize};
use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;

pub fn create_index_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: usize,
) -> Result<PoolBuffer> {
    let item_size = size_of::<u32>() as DeviceSize;
    
    let managed = buffer_factory.create_managed_buffer(
        "index_buffer",
        capacity as DeviceSize * item_size,
        BufferUsageFlags::INDEX_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
    )?;
    
    Ok(PoolBuffer::handle(managed, item_size))
}
