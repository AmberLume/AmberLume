use index_allocator::ResourceId;

pub struct TerrainGenerateRequest {
    pub mesh_id: ResourceId,

    pub cell_size: f32,

    pub heights: Vec<f32>,
}
