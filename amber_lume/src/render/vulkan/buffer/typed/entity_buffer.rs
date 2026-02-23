use crate::render::vulkan::factories::buffer::pool_buffer::PoolBuffer;
use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use gpu_allocator::MemoryLocation;
use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct EntityGpuData {
    pub transform_matrix: [[f32; 4]; 4],
    pub model_index: u32,
    _pad0: [f32; 3],
}

impl EntityGpuData {
    pub fn create(transform_matrix: Mat4, model_index: u32) -> Self {
        Self {
            transform_matrix: transform_matrix.to_cols_array_2d(),
            model_index,
            _pad0: [0.0; 3],
        }
    }
}

pub fn create_entity_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: u32,
) -> Result<PoolBuffer> {
    let item_size = size_of::<EntityGpuData>() as DeviceSize;
    
    let managed = buffer_factory.create_managed_buffer(
        "entity_buffer",
        capacity as DeviceSize * item_size,
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::CpuToGpu,
    )?;
    
    Ok(PoolBuffer::handle(managed, item_size, capacity))
}
