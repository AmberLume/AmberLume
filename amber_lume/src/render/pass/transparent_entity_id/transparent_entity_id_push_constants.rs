use gpu::BufferRange;
use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

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
        scene_buffer: BufferRange,
        draw_data_buffer: BufferRange,
        vertex_buffer: BufferRange,
        entity_buffer: BufferRange,
        bone_transform_buffer: BufferRange,
    ) -> Self {
        Self {
            scene_buffer_device_address: scene_buffer.device_address,
            draw_data_buffer_device_address: draw_data_buffer.device_address,
            vertex_buffer_device_address: vertex_buffer.device_address,
            entity_buffer_device_address: entity_buffer.device_address,
            bone_transform_buffer_device_address: bone_transform_buffer.device_address,
        }
    }
}
