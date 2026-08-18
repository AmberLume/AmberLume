use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct TerrainGenerateRequestGPU {
    pub vertex_offset: u32,
    pub height_offset: u32,
    pub cell_size: f32,
    pub level_deltas: u32,
}

impl TerrainGenerateRequestGPU {
    pub fn create(vertex_offset: u32, height_offset: u32, cell_size: f32, level_deltas: u32) -> Self {
        Self {
            vertex_offset,
            height_offset,
            cell_size,
            level_deltas,
        }
    }
}
