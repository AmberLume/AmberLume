use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use bytemuck::{Pod, Zeroable};
use gpu_allocator::MemoryLocation;
use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::vulkan::factories::buffer::pool_buffer::PoolBuffer;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct MaterialGpuData {
    pub base_color: [f32; 4],

    pub base_color_texture_index: u32,
    _pad0: [u32; 3],
}

impl MaterialGpuData {
    pub fn create(
        base_color: [f32; 4],
        base_color_texture_index: u32,
    ) -> Self {
        Self {
            base_color,

            base_color_texture_index,
            _pad0: [0; 3],
        }
    }
}

pub fn create_material_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: usize,
) -> Result<PoolBuffer> {
    let item_size = size_of::<MaterialGpuData>() as DeviceSize;
    
    let managed = buffer_factory.create_managed_buffer(
        "material_buffer",
        capacity as DeviceSize * item_size,
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::CpuToGpu,
    )?;
    
    Ok(PoolBuffer::handle(managed, item_size))
}
