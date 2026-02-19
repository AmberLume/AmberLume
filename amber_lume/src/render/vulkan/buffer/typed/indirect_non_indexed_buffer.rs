use anyhow::{Result};
use ash::vk::{BufferUsageFlags, DeviceSize};
use gpu_allocator::MemoryLocation;
use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::vulkan::factories::buffer::pool_buffer::PoolBuffer;

#[repr(C)]
pub struct IndirectNonIndexedGpuData {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: i32,
}

pub fn create_collider_indirect_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: usize,
) -> Result<PoolBuffer> {
    let item_size = size_of::<IndirectNonIndexedGpuData>() as DeviceSize;
    
    let managed = buffer_factory.create_managed_buffer(
        "collider_indirect_non_indexed_buffer",
        capacity as DeviceSize * item_size,
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::INDIRECT_BUFFER,
        MemoryLocation::GpuOnly,
    )?;
    
    Ok(PoolBuffer::handle(managed, item_size))
}
