use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct TLASInstancesPushConstants {
    pub entity_buffer_device_address: DeviceAddress,
    pub blas_address_buffer_device_address: DeviceAddress,
    pub instance_buffer_device_address: DeviceAddress,

    pub entity_count: u32,

    _pad0: [u32; 25],
}

impl TLASInstancesPushConstants {
    pub fn create(
        entity_buffer_device_address: DeviceAddress,
        blas_address_buffer_device_address: DeviceAddress,
        instance_buffer_device_address: DeviceAddress,
        entity_count: u32,
    ) -> Self {
        Self {
            entity_buffer_device_address,
            blas_address_buffer_device_address,
            instance_buffer_device_address,

            entity_count,

            _pad0: [0; 25],
        }
    }
}
