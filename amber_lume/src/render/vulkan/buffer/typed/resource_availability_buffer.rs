use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use gpu_allocator::MemoryLocation;
use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::vulkan::factories::buffer::pool_buffer::PoolBuffer;

pub fn create_resource_availability_buffer(
    buffer_factory: &ManagedBufferFactory,
    tag: &str,
    capacity: usize,
) -> Result<PoolBuffer> {
    let item_size = size_of::<u32>() as DeviceSize;
    
    let managed = buffer_factory.create_managed_buffer(
        &format!("{}_availability_buffer", tag),
        capacity as DeviceSize * item_size,
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::CpuToGpu,
    )?;
    
    Ok(PoolBuffer::handle(managed, item_size))
}
