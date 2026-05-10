use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct CascadeComputePushConstants {
    pub scene_buffer_device_address: DeviceAddress,
    pub sdsm_buffer_device_address: DeviceAddress,
    pub culling_view_buffer_device_address: DeviceAddress,
    pub shadow_cascades_buffer_device_address: DeviceAddress,

    pub cascade_count: u32,
    pub cascade_view_offset: u32,

    pub light_margin: f32,
    pub fallback_z_max: f32,
    pub split_lambda: f32,
    pub shadow_caster_extension: f32,

    _pad0: [u32; 18],
}

impl CascadeComputePushConstants {
    pub fn create(
        scene_buffer_device_address: DeviceAddress,
        sdsm_buffer_device_address: DeviceAddress,
        culling_view_buffer_device_address: DeviceAddress,
        shadow_cascades_buffer_device_address: DeviceAddress,
        cascade_count: u32,
        cascade_view_offset: u32,
        light_margin: f32,
        fallback_z_max: f32,
        split_lambda: f32,
        shadow_caster_extension: f32,
    ) -> Self {
        Self {
            scene_buffer_device_address,
            sdsm_buffer_device_address,
            culling_view_buffer_device_address,
            shadow_cascades_buffer_device_address,

            cascade_count,
            cascade_view_offset,

            light_margin,
            fallback_z_max,
            split_lambda,
            shadow_caster_extension,

            _pad0: [0; 18],
        }
    }
}
