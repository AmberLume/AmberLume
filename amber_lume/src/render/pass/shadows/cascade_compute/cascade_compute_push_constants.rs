use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use render_graph::PhysicalBuffer;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct CascadeComputePushConstants {
    pub scene_buffer_device_address: DeviceAddress,
    pub depth_reduce_result_buffer_device_address: DeviceAddress,
    pub culling_view_buffer_device_address: DeviceAddress,
    pub shadow_cascades_buffer_device_address: DeviceAddress,
    pub cascade_statistics_buffer_device_address: DeviceAddress,

    pub cascade_count: u32,
    pub shadow_resolution: u32,

    pub shadow_max_distance: f32,
    pub split_lambda: f32,
    pub shadow_caster_extension: f32,

    _pad0: [u32; 17],
}

impl CascadeComputePushConstants {
    pub fn create(
        scene_buffer: PhysicalBuffer,
        depth_reduce_result_buffer: PhysicalBuffer,
        culling_view_buffer: PhysicalBuffer,
        shadow_cascades_buffer: PhysicalBuffer,
        cascade_statistics_buffer_device_address: DeviceAddress,
        cascade_count: u32,
        shadow_resolution: u32,
        shadow_max_distance: f32,
        split_lambda: f32,
        shadow_caster_extension: f32,
    ) -> Self {
        Self {
            scene_buffer_device_address: scene_buffer.device_address,
            depth_reduce_result_buffer_device_address: depth_reduce_result_buffer.device_address,
            culling_view_buffer_device_address: culling_view_buffer.device_address,
            shadow_cascades_buffer_device_address: shadow_cascades_buffer.device_address,
            cascade_statistics_buffer_device_address,

            cascade_count,
            shadow_resolution,

            shadow_max_distance,
            split_lambda,
            shadow_caster_extension,

            _pad0: [0; 17],
        }
    }
}
