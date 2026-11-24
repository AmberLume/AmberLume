use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct TerrainStitchRequestGPU {
    pub mesh_id: u32,
    pub edge_height_offset: u32,
    pub level_deltas: u32,

    _pad0: u32,
}

impl TerrainStitchRequestGPU {
    pub fn create(mesh_id: u32, edge_height_offset: u32, level_deltas: u32) -> Self {
        Self {
            mesh_id,
            edge_height_offset,
            level_deltas,

            _pad0: 0,
        }
    }
}
