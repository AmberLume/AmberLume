use index_allocator::Allocation;
use index_allocator::ResourceId;
use terrain::ChunkPayload;

pub struct TerrainChunk {
    pub payload: Box<ChunkPayload>,

    pub mesh_id: ResourceId,
    pub vertices_allocation: Allocation,
    pub vertex_attributes_allocation: Allocation,
    pub submeshes_allocation: Allocation,

    pub level_deltas: [u32; 4],
}
