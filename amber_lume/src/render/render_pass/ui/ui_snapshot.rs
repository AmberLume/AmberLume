pub struct UiSnapshot {
    pub draw_layers: Vec<UiDrawLayer>,
}

pub struct UiDrawLayer {
    pub draw_calls: Vec<UiDrawCall>,
}

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
