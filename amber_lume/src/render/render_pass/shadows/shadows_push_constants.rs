use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ShadowsPushConstants {
    pub scene_buffer_device_address: DeviceAddress,
    pub draw_data_buffer_device_address: DeviceAddress,
    pub entity_buffer_device_address: DeviceAddress,
    pub vertex_buffer_device_address: DeviceAddress,

    pub shadow_cascade_index: u32,
    _pad0: u32,
}

impl ShadowsPushConstants {
    pub fn create(
        scene_buffer_device_address: DeviceAddress,
        draw_data_buffer_device_address: DeviceAddress,
        entity_buffer_device_address: DeviceAddress,
        vertex_buffer_device_address: DeviceAddress,
        shadow_cascade_index: u32,
    ) -> Self {
        Self {
            scene_buffer_device_address,
            draw_data_buffer_device_address,
            entity_buffer_device_address,
            vertex_buffer_device_address,

            shadow_cascade_index,
            _pad0: 0,
        }
    }
}
