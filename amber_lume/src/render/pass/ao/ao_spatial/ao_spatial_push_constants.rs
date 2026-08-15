use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C, align(8))]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct AoSpatialPushConstants {
    pub scene_buffer_device_address: DeviceAddress,

    pub noisy_descriptor_id: u32,
    pub guide_descriptor_id: u32,
    pub ao_storage_id: u32,
    pub width: u32,
    pub height: u32,

    pub plane_sensitivity: f32,
    pub normal_threshold: f32,
    pub blur_radius: f32,

    pub frame_number: u32,

    _pad0: [u32; 21],
}

impl AoSpatialPushConstants {
    pub fn create(
        scene_buffer_device_address: DeviceAddress,
        noisy_descriptor_id: u32,
        guide_descriptor_id: u32,
        ao_storage_id: u32,
        width: u32,
        height: u32,
        plane_sensitivity: f32,
        normal_threshold: f32,
        blur_radius: f32,
        frame_number: u32,
    ) -> Self {
        Self {
            scene_buffer_device_address,

            noisy_descriptor_id,
            guide_descriptor_id,
            ao_storage_id,
            width,
            height,

            plane_sensitivity,
            normal_threshold,
            blur_radius,

            frame_number,

            _pad0: [0; 21],
        }
    }
}
