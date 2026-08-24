use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::BufferRange;

#[repr(C, align(8))]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct RTShadowPushConstants {
    pub scene_buffer_device_address: DeviceAddress,

    pub depth_descriptor_id: u32,
    pub normal_descriptor_id: u32,
    pub visibility_storage_id: u32,
    pub tlas_descriptor_id: u32,

    pub sun_angular_radius: f32,
    pub sample_count: u32,
    pub frame_number: u32,

    _pad1: u32,
}

impl RTShadowPushConstants {
    pub fn create(
        scene_buffer: BufferRange,
        depth_descriptor_id: u32,
        normal_descriptor_id: u32,
        visibility_storage_id: u32,
        tlas_descriptor_id: u32,
        sun_angular_radius: f32,
        sample_count: u32,
        frame_number: u32,
    ) -> Self {
        Self {
            scene_buffer_device_address: scene_buffer.device_address,

            depth_descriptor_id,
            normal_descriptor_id,
            visibility_storage_id,
            tlas_descriptor_id,

            sun_angular_radius,
            sample_count,
            frame_number,

            _pad1: 0,
        }
    }
}
