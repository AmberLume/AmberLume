use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use crate::render::render_graph::virtual_buffer::physical_buffer::PhysicalBuffer;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ShadowsPushConstants {
    pub draw_data_buffer_device_address: DeviceAddress,
    pub entity_buffer_device_address: DeviceAddress,
    pub vertex_buffer_device_address: DeviceAddress,
    pub bone_transform_buffer_device_address: DeviceAddress,
    pub shadow_cascades_buffer_device_address: DeviceAddress,

    _pad0: [u32; 22],
}

impl ShadowsPushConstants {
    pub fn create(
        draw_data_buffer: &PhysicalBuffer,
        entity_buffer: &PhysicalBuffer,
        vertex_buffer_device_address: DeviceAddress,
        bone_transform_buffer_device_address: DeviceAddress,
        shadow_cascades_buffer: &PhysicalBuffer,
    ) -> Self {
        Self {
            draw_data_buffer_device_address: draw_data_buffer.device_address,
            entity_buffer_device_address: entity_buffer.device_address,
            vertex_buffer_device_address,
            bone_transform_buffer_device_address,
            shadow_cascades_buffer_device_address: shadow_cascades_buffer.device_address,

            _pad0: [0; 22],
        }
    }
}
