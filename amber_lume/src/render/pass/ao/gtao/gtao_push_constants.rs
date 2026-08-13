use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct GtaoPushConstants {
    pub scene_buffer_device_address: DeviceAddress,

    pub depth_descriptor_id: u32,
    pub normal_descriptor_id: u32,
    pub gtao_storage_id: u32,
    pub width: u32,
    pub height: u32,
    pub temporal_index: u32,

    pub radius: f32,
    pub power: f32,

    _pad0: [u32; 22],
}

impl GtaoPushConstants {
    pub fn create(
        scene_buffer_device_address: DeviceAddress,
        depth_descriptor_id: u32,
        normal_descriptor_id: u32,
        gtao_storage_id: u32,
        width: u32,
        height: u32,
        temporal_index: u32,
        radius: f32,
        power: f32,
    ) -> Self {
        Self {
            scene_buffer_device_address,

            depth_descriptor_id,
            normal_descriptor_id,
            gtao_storage_id,
            width,
            height,
            temporal_index,

            radius,
            power,

            _pad0: [0; 22],
        }
    }
}
