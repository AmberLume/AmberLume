use bytemuck::{Pod, Zeroable};
use gpu::ViewProjectionMatrix;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ShadowResolvePushConstants {
    pub inverse_view_projection: [[f32; 4]; 4],

    pub scene_buffer_device_address: u64,
    pub shadow_cascades_buffer_device_address: u64,

    pub depth_descriptor_id: u32,
    pub normal_descriptor_id: u32,
    pub shadow_array_descriptor_id: u32,
    pub output_storage_id: u32,

    pub shadow_pcf_sample_count: u32,
    pub frame_index: u32,

    pub shadow_bias: f32,
    pub shadow_normal_bias: f32,
    pub shadow_pcf_world_radius: f32,
    pub shadow_cascade_blend_range: f32,

    _pad0: [u32; 2],
}

impl ShadowResolvePushConstants {
    pub fn create(
        view_projection: &ViewProjectionMatrix,
        scene_buffer_device_address: u64,
        shadow_cascades_buffer_device_address: u64,
        depth_descriptor_id: u32,
        normal_descriptor_id: u32,
        shadow_array_descriptor_id: u32,
        output_storage_id: u32,
        shadow_pcf_sample_count: u32,
        frame_index: u32,
        shadow_bias: f32,
        shadow_normal_bias: f32,
        shadow_pcf_world_radius: f32,
        shadow_cascade_blend_range: f32,
    ) -> Self {
        Self {
            inverse_view_projection: view_projection.inverted().value.to_cols_array_2d(),

            scene_buffer_device_address,
            shadow_cascades_buffer_device_address,

            depth_descriptor_id,
            normal_descriptor_id,
            shadow_array_descriptor_id,
            output_storage_id,

            shadow_pcf_sample_count,
            frame_index,

            shadow_bias,
            shadow_normal_bias,
            shadow_pcf_world_radius,
            shadow_cascade_blend_range,

            _pad0: [0; 2],
        }
    }
}
