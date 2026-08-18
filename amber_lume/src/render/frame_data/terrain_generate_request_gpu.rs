use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct TerrainGenerateRequestGPU {
    pub vertex_offset: u32,
    pub height_offset: u32,

    _pad0: [u32; 2],
}

impl TerrainGenerateRequestGPU {
    pub fn create(vertex_offset: u32, height_offset: u32) -> Self {
        Self {
            vertex_offset,
            height_offset,

            _pad0: [0; 2],
        }
    }
}
