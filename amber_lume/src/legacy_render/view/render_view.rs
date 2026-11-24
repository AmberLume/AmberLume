use crate::legacy_render::view::view_matrix::ViewMatrix;
use crate::legacy_render::view::view_projection_matrix::ViewProjectionMatrix;
use math::Vec2;

#[derive(Clone, Copy)]
pub struct RenderView {
    pub view_projection: ViewProjectionMatrix,
    pub previous_view_projection: ViewProjectionMatrix,
    
    pub view: ViewMatrix,

    pub tan_half_fov: Vec2,

    pub jitter: Vec2,

    pub mip_bias: f32,
}
