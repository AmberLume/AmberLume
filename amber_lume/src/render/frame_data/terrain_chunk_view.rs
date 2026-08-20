use glam::Vec3;
use index_allocator::ResourceId;

pub struct TerrainChunkView {
    pub center: Vec3,
    pub level: u32,

    pub mesh_id: ResourceId,
}
