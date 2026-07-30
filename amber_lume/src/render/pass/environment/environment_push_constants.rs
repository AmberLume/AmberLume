use bytemuck::{Pod, Zeroable};
use gpu::ViewProjectionMatrix;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct EnvironmentPushConstants {
    pub inverse_view_projection: [[f32; 4]; 4],

    pub sun_direction: [f32; 3],
    pub time: f32,

    _pad0: [u32; 12],
}

impl EnvironmentPushConstants {
    pub fn create(
        view_projection: &ViewProjectionMatrix,
        sun_direction: [f32; 3],
        time: f32,
    ) -> Self {
        Self {
            inverse_view_projection: view_projection.inverted().value.to_cols_array_2d(),

            sun_direction,
            time,

            _pad0: [0; 12],
        }
    }
}
