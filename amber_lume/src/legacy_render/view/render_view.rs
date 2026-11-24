use gpu::{ViewMatrix, ViewProjectionMatrix};
use glam::Vec2;

#[derive(Clone, Copy)]
pub struct RenderView {
    pub view_projection: ViewProjectionMatrix,
    pub previous_view_projection: ViewProjectionMatrix,
    
    pub view: ViewMatrix,

    pub tan_half_fov: Vec2,

    pub jitter: [f32; 2],

    pub mip_bias: f32,
}
