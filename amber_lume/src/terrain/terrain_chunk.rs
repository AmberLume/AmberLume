use resource_residency::ResRef;
use shipyard::EntityId;
use std::sync::Arc;

pub struct TerrainChunk {
    pub entity: EntityId,

    pub handle: Arc<ResRef>,

    pub vertex_offset: u32,
    pub level_deltas: [u32; 4],
    pub traced: bool,
}
