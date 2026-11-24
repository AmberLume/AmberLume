use bytemuck::{Pod, Zeroable};
use math::{Mat4, Vec2, Vec3};
use crate::legacy_render::view::view_matrix::ViewMatrix;
use crate::legacy_render::view::view_projection_matrix::ViewProjectionMatrix;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct CameraGPU {
    pub view_projection: Mat4,
    pub previous_view_projection: Mat4,
    pub inverse_view_projection: Mat4,

    pub view: Mat4,

    pub position: Vec3,
    _pad0: u32,

    pub tan_half_fov: Vec2,
    pub near: f32,
    pub far: f32,

    pub jitter: Vec2,
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
        jitter: Vec2,
        mip_bias: f32,
    ) -> Self {
        Self {
            view_projection: view_projection.value,
            previous_view_projection: previous_view_projection.value,
            inverse_view_projection: inverse_view_projection.value,

            view: view.value,

            position,
            _pad0: 0,

            tan_half_fov,
            near,
            far,

            jitter,
            mip_bias,
            _pad1: 0,
        }
    }
}
