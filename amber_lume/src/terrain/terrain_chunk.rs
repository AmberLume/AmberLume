use resource_residency::ResRef;
use std::sync::Arc;
use terrain::ChunkPayload;

pub struct TerrainChunk {
    pub payload: Box<ChunkPayload>,

    pub mesh: Arc<ResRef>,

    pub level_deltas: [u32; 4],
}
