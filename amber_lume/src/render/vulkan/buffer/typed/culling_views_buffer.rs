use crate::render::vulkan::factories::buffer::pool_buffer::PoolBuffer;
use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceAddress, DeviceSize};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec4Swizzles};
use gpu_allocator::MemoryLocation;
use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct CullingViewGpuData {
    pub frustum_planes: [[f32; 4]; 6],

    pub indirect_buffer_device_address: DeviceAddress,
    pub draw_count_buffer_device_address: DeviceAddress,
    pub draw_data_buffer_device_address: DeviceAddress,

    _pad0: [u32; 2],
}

impl CullingViewGpuData {
    pub fn create(
        transform_matrix: Mat4,
        indirect_buffer_device_address: DeviceAddress,
        draw_count_buffer_device_address: DeviceAddress,
        draw_data_buffer_device_address: DeviceAddress,
    ) -> Self {
        let frustum_planes = Self::frustum_planes_from_matrix(transform_matrix);

        Self {
            frustum_planes,

            indirect_buffer_device_address,
            draw_count_buffer_device_address,
            draw_data_buffer_device_address,

            _pad0: [0; 2],
        }
    }

    fn frustum_planes_from_matrix(matrix: Mat4) -> [[f32; 4]; 6] {
        let mut planes = [[0.0f32; 4]; 6];

        let combinations = [
            matrix.row(3) + matrix.row(0),
            matrix.row(3) - matrix.row(0),
            matrix.row(3) + matrix.row(1),
            matrix.row(3) - matrix.row(1),
            matrix.row(2),
            matrix.row(3) - matrix.row(2),
        ];

        for (index, plane) in combinations.iter().enumerate() {
            let length = plane.xyz().length();

            let normalized = plane / length;

            planes[index] = normalized.to_array();
        }

        planes
    }
}

pub fn create_culling_views_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: u32,
) -> Result<PoolBuffer> {
    let item_size = size_of::<CullingViewGpuData>() as DeviceSize;

    let managed = buffer_factory.create_managed_buffer(
        "culling_views_buffer",
        capacity as DeviceSize * item_size,
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::CpuToGpu,
    )?;

    Ok(PoolBuffer::handle(managed, item_size, capacity))
}
