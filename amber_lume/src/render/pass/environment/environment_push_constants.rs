use bytemuck::{Pod, Zeroable};
use crate::utils::matrix_wrappers::ViewProjectionMatrix;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct EnvironmentPushConstants {
    pub inverse_view_projection: [[f32; 4]; 4],

    pub time: f32,

    pub frequency: f32,
    pub probability: f32,
    pub core_size: f32,

    pub color_cool: [f32; 4],
    pub color_warm: [f32; 4],

    _pad0: [u32; 4],
}

impl EnvironmentPushConstants {
    pub fn create(
        view_projection: &ViewProjectionMatrix,
        time: f32,
        frequency: f32,
        probability: f32,
        core_size: f32,
        color_cool: [f32; 3],
        color_warm: [f32; 3],
    ) -> Self {
        Self {
            inverse_view_projection: view_projection.inverted().value.to_cols_array_2d(),

            time,

            frequency,
            probability,
            core_size,

            color_cool: [color_cool[0], color_cool[1], color_cool[2], 0.0],
            color_warm: [color_warm[0], color_warm[1], color_warm[2], 0.0],

            _pad0: [0; 4],
        }
    }
}
