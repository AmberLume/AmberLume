use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct GtaoPushConstants {
    pub scene_buffer_device_address: DeviceAddress,

    pub pyramid_descriptor_id: u32,
    pub normal_descriptor_id: u32,
    pub gtao_storage_id: u32,
    pub width: u32,
    pub height: u32,

    pub radius: f32,
    pub power: f32,

    _pad0: [u32; 23],
}

impl GtaoPushConstants {
    pub fn create(
        scene_buffer_device_address: DeviceAddress,
        pyramid_descriptor_id: u32,
        normal_descriptor_id: u32,
        gtao_storage_id: u32,
        width: u32,
        height: u32,
        radius: f32,
        power: f32,
    ) -> Self {
        Self {
            scene_buffer_device_address,

            pyramid_descriptor_id,
            normal_descriptor_id,
            gtao_storage_id,
            width,
            height,

            radius,
            power,

            _pad0: [0; 23],
        }
    }
}
