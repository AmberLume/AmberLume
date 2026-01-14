pub struct UiSnapshot {
    pub draw_calls: Vec<UiDrawCall>
}

pub struct UiDrawCall {
    pub index_count: usize,
    pub index_offset: usize,
    pub vertex_offset: usize,
}
