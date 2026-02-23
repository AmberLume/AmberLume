use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use bytemuck::{Pod, Zeroable};
use gpu_allocator::MemoryLocation;
use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::vulkan::factories::buffer::pool_buffer::PoolBuffer;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct ModelGpuData {
    pub submesh_offset: u32,
    pub submesh_count: u32,
    _pad0: [u32; 2],
}

impl ModelGpuData {
    pub fn create(
        submesh_offset: u32,
        submesh_count: u32,
    ) -> Self {
        Self {
            submesh_offset,
            submesh_count,
            _pad0: [0; 2],
        }
    }
}

pub fn create_model_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: u32,
) -> Result<PoolBuffer> {
    let item_size = size_of::<ModelGpuData>() as DeviceSize;
    
    let managed = buffer_factory.create_managed_buffer(
        "model_buffer",
        capacity as DeviceSize * item_size,
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
    )?;
    
    Ok(PoolBuffer::handle(managed, item_size, 1))
}
