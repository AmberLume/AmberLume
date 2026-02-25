use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::vulkan::factories::buffer::pool_buffer::PoolBuffer;
use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use bytemuck::{Pod, Zeroable};
use gpu_allocator::MemoryLocation;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct ShadowCascadeGpuData {
    pub screen_to_light: [[f32; 4]; 4],
    pub split: f32,

    _pad0: [u32; 3],
}

impl ShadowCascadeGpuData {
    pub fn new(
        screen_to_light: [[f32; 4]; 4],
        split: f32,
    ) -> Self {
        Self {
            screen_to_light,
            split,

            _pad0: [0; 3],
        }
    }
}

impl Default for ShadowCascadeGpuData {
    fn default() -> Self {
        Self {
            screen_to_light: [[0.0; 4]; 4],
            split: 0.0,

            _pad0: [0; 3],
        }
    }
}

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct ShadowGpuData {
    pub cascades: [ShadowCascadeGpuData; 4],
}

impl ShadowGpuData {
    pub fn create(
        cascades: [ShadowCascadeGpuData; 4],
    ) -> Self {
        Self {
            cascades,
        }
    }
}

pub fn create_shadow_buffer(
    managed_buffer_factory: &ManagedBufferFactory,
) -> Result<PoolBuffer> {
    let item_size = size_of::<ShadowGpuData>() as DeviceSize;

    let managed_buffer = managed_buffer_factory.create_managed_buffer(
        "shadow_buffer",
        item_size as DeviceSize,
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::CpuToGpu,
    )?;

    Ok(PoolBuffer::handle(managed_buffer, item_size, 1))
}
