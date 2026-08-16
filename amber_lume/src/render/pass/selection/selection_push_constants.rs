use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C, align(8))]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct SelectionPushConstants {
    pub scene_buffer_device_address: DeviceAddress,
    pub entity_buffer_device_address: DeviceAddress,

    pub entity_id_texel_scale: [f32; 2],

    pub entity_id_texture: u32,
    pub mask_texture: u32,

    pub radius: i32,
    pub mask_scale: i32,

    _pad0: [u32; 22],
}

impl SelectionPushConstants {
    pub fn create(
        scene_buffer_device_address: DeviceAddress,
        entity_buffer_device_address: DeviceAddress,
        entity_id_texel_scale: [f32; 2],
        entity_id_texture: u32,
        mask_texture: u32,
        radius: i32,
        mask_scale: i32,
    ) -> Self {
        Self {
            scene_buffer_device_address,
            entity_buffer_device_address,

            entity_id_texel_scale,

            entity_id_texture,
            mask_texture,

            radius,
            mask_scale,

            _pad0: [0; 22],
        }
    }
}
