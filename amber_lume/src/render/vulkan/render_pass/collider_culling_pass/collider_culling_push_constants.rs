use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ColliderCullingPushConstants {
    pub collider_indirect_buffer_device_address: DeviceAddress,
    pub draw_count_buffer_device_address: DeviceAddress,
    pub collider_buffer_device_address: DeviceAddress,

    pub collider_count: u32,

    _pad0: u32,
}

impl ColliderCullingPushConstants {
    pub fn create(
        collider_indirect_buffer_device_address: DeviceAddress,
        draw_count_buffer_device_address: DeviceAddress,
        collider_buffer_device_address: DeviceAddress,
        collider_count: u32,
    ) -> Self {
        Self {
            collider_indirect_buffer_device_address,
            draw_count_buffer_device_address,
            collider_buffer_device_address,

            collider_count,

            _pad0: 0,
        }
    }
}
