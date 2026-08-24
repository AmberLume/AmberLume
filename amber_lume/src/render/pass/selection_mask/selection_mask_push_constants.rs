use gpu::BufferRange;
use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C, align(8))]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct SelectionMaskPushConstants {
    pub entity_buffer_device_address: DeviceAddress,

    pub entity_id_texture: u32,
    pub mask_storage_id: u32,
    pub width: u32,
    pub height: u32,

    pub mask_scale: i32,

    _pad0: [u32; 25],
}

impl SelectionMaskPushConstants {
    pub fn create(
        entity_buffer: BufferRange,
        entity_id_texture: u32,
        mask_storage_id: u32,
        width: u32,
        height: u32,
        mask_scale: i32,
    ) -> Self {
        Self {
            entity_buffer_device_address: entity_buffer.device_address,

            entity_id_texture,
            mask_storage_id,
            width,
            height,

            mask_scale,

            _pad0: [0; 25],
        }
    }
}
