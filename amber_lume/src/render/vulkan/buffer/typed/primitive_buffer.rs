use crate::render::vulkan::factories::buffer::pool_buffer::PoolBuffer;
use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use bytemuck::{Pod, Zeroable};
use gpu_allocator::MemoryLocation;
use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct PrimitiveGpuData {
    pub index_offset: u32,
    pub index_count: u32,
    pub vertex_offset: u32,
    
    pub material_index: u32,
}

impl PrimitiveGpuData {
    pub fn create(
        index_count: u32,
        index_offset: u32,
        vertex_offset: u32,
        material_index: u32,
    ) -> Self {
        Self {
            index_offset,
            index_count,
            vertex_offset,
            material_index,
        }
    }
}

pub fn create_primitive_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: usize,
) -> Result<PoolBuffer> {
    let item_size = size_of::<PrimitiveGpuData>() as DeviceSize;

    let managed = buffer_factory.create_managed_buffer(
        "primitive_buffer",
        capacity as DeviceSize * item_size,
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::CpuToGpu,
    )?;

    Ok(PoolBuffer::handle(managed, item_size))
}
