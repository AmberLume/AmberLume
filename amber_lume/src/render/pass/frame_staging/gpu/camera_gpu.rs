use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3};
use gpu::{ViewMatrix, ViewProjectionMatrix};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct CameraGPU {
    pub view_projection: [[f32; 4]; 4],
    pub previous_view_projection: [[f32; 4]; 4],
    pub inverse_view_projection: [[f32; 4]; 4],

    pub view: [[f32; 4]; 4],

    pub position: [f32; 3],
    _pad0: u32,

    pub tan_half_fov: [f32; 2],
    pub near: f32,
    pub far: f32,

    pub jitter: [f32; 2],
    pub mip_bias: f32,
    _pad1: u32,
}

impl CameraGPU {
    pub fn new(
        view_projection: &ViewProjectionMatrix,
        previous_view_projection: &ViewProjectionMatrix,
        inverse_view_projection: &ViewProjectionMatrix,
        view: &ViewMatrix,
        position: Vec3,
        near: f32,
        far: f32,
        tan_half_fov: Vec2,
        jitter: [f32; 2],
        mip_bias: f32,
    ) -> Self {
        Self {
            view_projection: view_projection.value.to_cols_array_2d(),
            previous_view_projection: previous_view_projection.value.to_cols_array_2d(),
            inverse_view_projection: inverse_view_projection.value.to_cols_array_2d(),

            view: view.value.to_cols_array_2d(),

            position: position.to_array(),
            _pad0: 0,

            tan_half_fov: tan_half_fov.to_array(),
            near,
            far,

            jitter,
            mip_bias,
            _pad1: 0,
        }
    }
}
