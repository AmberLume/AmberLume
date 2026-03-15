use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct DepthPushConstants {
    pub scene_buffer_device_address: DeviceAddress,
    pub draw_data_buffer_device_address: DeviceAddress,
    pub entity_buffer_device_address: DeviceAddress,
    pub vertex_buffer_device_address: DeviceAddress,
}

impl DepthPushConstants {
    pub fn create(
        scene_buffer_device_address: DeviceAddress,
        draw_data_buffer_device_address: DeviceAddress,
        entity_buffer_device_address: DeviceAddress,
        vertex_buffer_device_address: DeviceAddress,
    ) -> Self {
        Self {
            scene_buffer_device_address,
            draw_data_buffer_device_address,
            entity_buffer_device_address,
            vertex_buffer_device_address,
        }
    }
}
