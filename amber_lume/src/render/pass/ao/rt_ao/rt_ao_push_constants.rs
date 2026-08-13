use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C, align(8))]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct RTAOPushConstants {
    pub scene_buffer_device_address: DeviceAddress,

    pub depth_descriptor_id: u32,
    pub normal_descriptor_id: u32,
    pub ao_storage_id: u32,
    pub width: u32,
    pub height: u32,
    pub tlas_descriptor_id: u32,

    pub ao_radius: f32,
    pub sample_count: u32,
    pub ao_power: f32,
    pub frame_number: u32,
    pub trace_period: u32,

    _pad0: [u32; 19],
}

impl RTAOPushConstants {
    pub fn create(
        scene_buffer_device_address: DeviceAddress,
        depth_descriptor_id: u32,
        normal_descriptor_id: u32,
        ao_storage_id: u32,
        width: u32,
        height: u32,
        tlas_descriptor_id: u32,
        ao_radius: f32,
        sample_count: u32,
        ao_power: f32,
        frame_number: u32,
        trace_period: u32,
    ) -> Self {
        Self {
            scene_buffer_device_address,

            depth_descriptor_id,
            normal_descriptor_id,
            ao_storage_id,
            width,
            height,
            tlas_descriptor_id,

            ao_radius,
            sample_count,
            ao_power,
            frame_number,
            trace_period,

            _pad0: [0; 19],
        }
    }
}
