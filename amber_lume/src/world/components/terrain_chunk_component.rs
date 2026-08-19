use resource_residency::ResRef;
use shipyard::Component;
use std::sync::Arc;
use terrain::ChunkCoordinate;

#[derive(Component)]
pub struct TerrainChunkComponent {
    pub coordinate: ChunkCoordinate,

    pub handle: Arc<ResRef>,

    pub vertex_offset: u32,
    pub level_deltas: [u32; 4],
    pub traced: bool,
}
