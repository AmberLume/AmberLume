use gpu::{ViewMatrix, ViewProjectionMatrix};
use glam::Vec2;

#[derive(Clone, Copy)]
pub struct RenderView {
    pub view_projection: ViewProjectionMatrix,
    pub previous_view_projection: ViewProjectionMatrix,
    pub jittered_view_projection: ViewProjectionMatrix,
    
    pub view: ViewMatrix,

    pub ndc_to_view_mul: Vec2,
    pub ndc_to_view_add: Vec2,

    pub jitter: [f32; 2],

    pub mip_bias: f32,
}

#[derive(Clone, Copy)]
pub struct RenderViewsLayout {
    pub main: RenderView,
    pub cascade_count: u32,
}
