use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::BufferRange;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct TLASInstancesPushConstants {
    pub entity_buffer_device_address: DeviceAddress,
    pub blas_address_buffer_device_address: DeviceAddress,
    pub instance_buffer_device_address: DeviceAddress,
    pub mesh_buffer_device_address: DeviceAddress,
    pub submesh_buffer_device_address: DeviceAddress,
    pub material_buffer_device_address: DeviceAddress,

    pub entity_count: u32,

    _pad0: [u32; 19],
}

impl TLASInstancesPushConstants {
    pub fn create(
        entity_buffer: BufferRange,
        blas_address_buffer: BufferRange,
        instance_buffer: BufferRange,
        mesh_buffer: BufferRange,
        submesh_buffer: BufferRange,
        material_buffer: BufferRange,
        entity_count: u32,
    ) -> Self {
        Self {
            entity_buffer_device_address: entity_buffer.device_address,
            blas_address_buffer_device_address: blas_address_buffer.device_address,
            instance_buffer_device_address: instance_buffer.device_address,
            mesh_buffer_device_address: mesh_buffer.device_address,
            submesh_buffer_device_address: submesh_buffer.device_address,
            material_buffer_device_address: material_buffer.device_address,

            entity_count,

            _pad0: [0; 19],
        }
    }
}
