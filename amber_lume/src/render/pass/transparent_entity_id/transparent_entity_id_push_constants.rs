use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::BufferRange;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct TransparentEntityIdPushConstants {
    pub camera_buffer_device_address: DeviceAddress,
    pub draw_data_buffer_device_address: DeviceAddress,
    pub entity_buffer_device_address: DeviceAddress,
    pub entity_motion_buffer_device_address: DeviceAddress,
}

impl TransparentEntityIdPushConstants {
    pub fn create(
        camera_buffer: BufferRange,
        draw_data_buffer: BufferRange,
        entity_buffer: BufferRange,
        entity_motion_buffer: BufferRange,
    ) -> Self {
        Self {
            camera_buffer_device_address: camera_buffer.device_address,
            draw_data_buffer_device_address: draw_data_buffer.device_address,
            entity_buffer_device_address: entity_buffer.device_address,
            entity_motion_buffer_device_address: entity_motion_buffer.device_address,
        }
    }
}
