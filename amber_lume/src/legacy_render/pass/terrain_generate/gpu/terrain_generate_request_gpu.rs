use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct TerrainGenerateRequestGPU {
    pub mesh_id: u32,
    pub height_offset: u32,
    pub cell_size: f32,

    _pad0: u32,
}

impl TerrainGenerateRequestGPU {
    pub fn create(mesh_id: u32, height_offset: u32, cell_size: f32) -> Self {
        Self {
            mesh_id,
            height_offset,
            cell_size,

            _pad0: 0,
        }
    }
}
