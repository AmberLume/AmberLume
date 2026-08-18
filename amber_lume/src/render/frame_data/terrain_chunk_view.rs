use glam::Vec3;

pub struct TerrainChunkView {
    pub center: Vec3,
    pub level: u32,

    pub vertex_offset: u32,
}
