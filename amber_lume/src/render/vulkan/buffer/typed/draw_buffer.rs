use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use bytemuck::{Pod, Zeroable};
use gpu_allocator::MemoryLocation;
use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::vulkan::factories::buffer::pool_buffer::PoolBuffer;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct DrawGpuData {
    pub entity_index: u32,
    pub primitive_index: u32,
    _pad: [u32; 2],
}

pub fn create_draw_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: usize,
) -> Result<PoolBuffer> {
    let item_size = size_of::<DrawGpuData>() as DeviceSize;
    
    let managed = buffer_factory.create_managed_buffer(
        "draw_buffer",
        capacity as DeviceSize * item_size,
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        MemoryLocation::GpuOnly,
    )?;
    
    Ok(PoolBuffer::handle(managed, item_size))
}
