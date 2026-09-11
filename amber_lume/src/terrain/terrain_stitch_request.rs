use index_allocator::ResourceId;

pub struct TerrainStitchRequest {
    pub mesh_id: ResourceId,

    pub level_deltas: u32,

    pub edge_heights: Vec<f32>,
}
