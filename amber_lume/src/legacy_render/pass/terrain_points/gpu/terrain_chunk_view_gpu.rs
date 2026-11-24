use bytemuck::{Pod, Zeroable};
use math::Vec3;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct TerrainChunkViewGPU {
    pub center: Vec3,
    pub level: u32,

    pub mesh_id: u32,

    _pad0: [u32; 3],
}

impl TerrainChunkViewGPU {
    pub fn create(center: Vec3, level: u32, mesh_id: u32) -> Self {
        Self {
            center,
            level,

            mesh_id,

            _pad0: [0; 3],
        }
    }
}
