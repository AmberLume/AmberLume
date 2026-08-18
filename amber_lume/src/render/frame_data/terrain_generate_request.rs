pub struct TerrainGenerateRequest {
    pub vertex_offset: u32,
    pub cell_size: f32,
    pub level_deltas: u32,

    pub heights: Vec<f32>,
}
