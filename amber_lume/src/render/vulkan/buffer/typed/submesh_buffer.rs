use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::vulkan::factories::buffer::pool_buffer::PoolBuffer;
use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use bytemuck::{Pod, Zeroable};
use gpu_allocator::MemoryLocation;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct SubmeshGpuData {
    pub index_offset: u32,
    pub index_count: u32,
    pub vertex_offset: u32,

    pub material_index: u32,

    pub bounds_min: [f32; 4],
    pub bounds_max: [f32; 4],
}

impl SubmeshGpuData {
    pub fn create(
        index_count: u32,
        index_offset: u32,
        vertex_offset: u32,
        material_index: u32,
        bounds: [f32; 6],
    ) -> Self {
        Self {
            index_offset,
            index_count,
            vertex_offset,
            material_index,
            bounds_min: [bounds[0], bounds[1], bounds[2], 0.0],
            bounds_max: [bounds[3], bounds[4], bounds[5], 0.0],
        }
    }
}

pub fn create_submesh_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: usize,
) -> Result<PoolBuffer> {
    let item_size = size_of::<SubmeshGpuData>() as DeviceSize;

    let managed = buffer_factory.create_managed_buffer(
        "submesh_buffer",
        capacity as DeviceSize * item_size,
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
    )?;

    Ok(PoolBuffer::handle(managed, item_size))
}
