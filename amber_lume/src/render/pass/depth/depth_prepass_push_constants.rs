use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::BufferRange;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct DepthPrepassPushConstants {
    pub camera_buffer_device_address: DeviceAddress,
    pub draw_data_buffer_device_address: DeviceAddress,
    pub entity_buffer_device_address: DeviceAddress,
    pub vertex_position_buffer_device_address: DeviceAddress,
    pub vertex_normal_tangent_buffer_device_address: DeviceAddress,

    _pad0: [u32; 22],
}

impl DepthPrepassPushConstants {
    pub fn create(
        camera_buffer: BufferRange,
        draw_data_buffer: BufferRange,
        entity_buffer: BufferRange,
        vertex_position_buffer: BufferRange,
        vertex_normal_tangent_buffer: BufferRange,
    ) -> Self {
        Self {
            camera_buffer_device_address: camera_buffer.device_address,
            draw_data_buffer_device_address: draw_data_buffer.device_address,
            entity_buffer_device_address: entity_buffer.device_address,
            vertex_position_buffer_device_address: vertex_position_buffer.device_address,
            vertex_normal_tangent_buffer_device_address: vertex_normal_tangent_buffer.device_address,

            _pad0: [0; 22],
        }
    }
}
