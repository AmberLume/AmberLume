use bytemuck::{Pod, Zeroable};
use crate::utils::matrix_wrappers::ViewProjectionMatrix;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct RTShadowPushConstants {
    pub inverse_view_projection: [[f32; 4]; 4],

    pub sun_direction: [f32; 4],

    pub depth_descriptor_id: u32,
    pub normal_descriptor_id: u32,
    pub visibility_storage_id: u32,
    pub width: u32,
    pub height: u32,
    pub tlas_descriptor_id: u32,

    pub sun_angular_radius: f32,
    pub sample_count: u32,
    pub frame_number: u32,

    _pad0: [u32; 3],
}

impl RTShadowPushConstants {
    pub fn create(
        view_projection: &ViewProjectionMatrix,
        sun_direction: [f32; 3],
        depth_descriptor_id: u32,
        normal_descriptor_id: u32,
        visibility_storage_id: u32,
        width: u32,
        height: u32,
        tlas_descriptor_id: u32,
        sun_angular_radius: f32,
        sample_count: u32,
        frame_number: u32,
    ) -> Self {
        Self {
            inverse_view_projection: view_projection.inverted().value.to_cols_array_2d(),

            sun_direction: [sun_direction[0], sun_direction[1], sun_direction[2], 0.0],

            depth_descriptor_id,
            normal_descriptor_id,
            visibility_storage_id,
            width,
            height,
            tlas_descriptor_id,

            sun_angular_radius,
            sample_count,
            frame_number,

            _pad0: [0; 3],
        }
    }
}
