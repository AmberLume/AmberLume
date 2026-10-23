use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::BufferRange;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct SelectionPushConstants {
    pub camera_buffer_device_address: DeviceAddress,
    pub entity_outline_buffer_device_address: DeviceAddress,

    pub entity_id_texel_scale: [f32; 2],

    pub entity_id_descriptor_id: u32,
    pub mask_descriptor_id: u32,

    pub radius: i32,
    pub mask_scale: i32,

    _pad0: [u32; 22],
}

impl SelectionPushConstants {
    pub fn create(
        camera_buffer: BufferRange,
        entity_outline_buffer: BufferRange,
        entity_id_texel_scale: [f32; 2],
        entity_id_descriptor_id: u32,
        mask_descriptor_id: u32,
        radius: i32,
        mask_scale: i32,
    ) -> Self {
        Self {
            camera_buffer_device_address: camera_buffer.device_address,
            entity_outline_buffer_device_address: entity_outline_buffer.device_address,

            entity_id_texel_scale,

            entity_id_descriptor_id,
            mask_descriptor_id,

            radius,
            mask_scale,

            _pad0: [0; 22],
        }
    }
}
