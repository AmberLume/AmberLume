use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use crate::render::render_graph::virtual_buffer::physical_buffer::PhysicalBuffer;

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct CascadeComputePushConstants {
    pub scene_buffer_device_address: DeviceAddress,
    pub sdsm_buffer_device_address: DeviceAddress,
    pub culling_view_buffer_device_address: DeviceAddress,
    pub shadow_cascades_buffer_device_address: DeviceAddress,
    pub cascade_statistics_buffer_device_address: DeviceAddress,

    pub cascade_count: u32,
    pub shadow_resolution: u32,

    pub fallback_z_max: f32,
    pub split_lambda: f32,
    pub shadow_caster_extension: f32,

    _pad0: [u32; 17],
}

impl CascadeComputePushConstants {
    pub fn create(
        scene_buffer: PhysicalBuffer,
        sdsm_buffer: PhysicalBuffer,
        culling_view_buffer: PhysicalBuffer,
        shadow_cascades_buffer: PhysicalBuffer,
        cascade_statistics_buffer_device_address: DeviceAddress,
        cascade_count: u32,
        shadow_resolution: u32,
        fallback_z_max: f32,
        split_lambda: f32,
        shadow_caster_extension: f32,
    ) -> Self {
        Self {
            scene_buffer_device_address: scene_buffer.device_address,
            sdsm_buffer_device_address: sdsm_buffer.device_address,
            culling_view_buffer_device_address: culling_view_buffer.device_address,
            shadow_cascades_buffer_device_address: shadow_cascades_buffer.device_address,
            cascade_statistics_buffer_device_address,

            cascade_count,
            shadow_resolution,

            fallback_z_max,
            split_lambda,
            shadow_caster_extension,

            _pad0: [0; 17],
        }
    }
}
