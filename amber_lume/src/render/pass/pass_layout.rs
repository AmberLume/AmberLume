use crate::ids::ChunkIndex;
use crate::utils::matrix_wrappers::{ViewMatrix, ViewProjectionMatrix};
use glam::Vec2;

pub struct RenderView {
    pub view_projection: ViewProjectionMatrix,
    pub previous_view_projection: ViewProjectionMatrix,
    
    pub view: ViewMatrix,

    pub ndc_to_view_mul: Vec2,
    pub ndc_to_view_add: Vec2,
}

pub struct RenderViewsLayout {
    pub main: RenderView,
    pub cascade_count: u32,
}

impl RenderViewsLayout {
    pub fn get_main_index() -> ChunkIndex { ChunkIndex { value: 0 } }

    pub fn get_shadow_index() -> ChunkIndex { ChunkIndex { value: 1 } }

    pub fn render_chunk_count() -> u32 { 2 }
}
