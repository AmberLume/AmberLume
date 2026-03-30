use crate::render::buffer::typed::ui_vertex_buffer::UiVertex;

#[derive(Clone)]
pub struct UiSnapshot {
    pub indices: Vec<u32>,
    pub vertices: Vec<UiVertex>,

    pub draw_layers: Vec<UiDrawLayer>,
}

#[derive(Clone)]
pub struct UiDrawLayer {
    pub draw_calls: Vec<UiDrawCall>,
}

#[derive(Clone)]
pub struct UiDrawCall {
    pub index_count: usize,
    pub index_offset: usize,
    pub vertex_offset: usize,
    
    pub clip: Option<ClipArea>,
    
    pub texture_index: u32,
    pub render_mode: RenderMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipArea {
    pub position: [i32; 2],
    pub size: [u32; 2],
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Solid = 0,
    Texture = 1,
}
