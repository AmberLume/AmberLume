use crate::clip_area::ClipArea;
use crate::render_mode::RenderMode;

#[derive(Clone)]
pub struct UiDrawCall {
    pub index_count: usize,
    pub index_offset: usize,
    pub vertex_offset: usize,
    
    pub clip: Option<ClipArea>,
    
    pub texture_index: u32,
    pub render_mode: RenderMode,
}
