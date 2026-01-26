pub struct UiSnapshot {
    pub draw_calls: Vec<UiDrawCall>
}

pub struct UiDrawCall {
    pub index_count: usize,
    pub index_offset: usize,
    pub vertex_offset: usize,
    
    pub texture_index: u32,
    pub render_mode: RenderMode,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Solid = 0,
    Texture = 1,
}
