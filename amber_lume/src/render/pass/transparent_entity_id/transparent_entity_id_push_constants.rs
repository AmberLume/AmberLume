use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use render_graph::PhysicalBuffer;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct TransparentEntityIdPushConstants {
    pub scene_buffer_device_address: DeviceAddress,
    pub draw_data_buffer_device_address: DeviceAddress,
    pub vertex_buffer_device_address: DeviceAddress,
    pub entity_buffer_device_address: DeviceAddress,
    pub bone_transform_buffer_device_address: DeviceAddress,
}

impl TransparentEntityIdPushConstants {
    pub fn create(
        scene_buffer: &PhysicalBuffer,
        draw_data_buffer: &PhysicalBuffer,
        vertex_buffer_device_address: DeviceAddress,
        entity_buffer: &PhysicalBuffer,
        bone_transform_buffer: &PhysicalBuffer,
    ) -> Self {
        Self {
            scene_buffer_device_address: scene_buffer.device_address,
            draw_data_buffer_device_address: draw_data_buffer.device_address,
            vertex_buffer_device_address,
            entity_buffer_device_address: entity_buffer.device_address,
            bone_transform_buffer_device_address: bone_transform_buffer.device_address,
        }
    }
}
