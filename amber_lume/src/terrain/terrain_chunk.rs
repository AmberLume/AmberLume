use resource_residency::ResRef;
use std::sync::Arc;
use terrain::{ChunkCoordinate, ChunkPayload};

pub struct TerrainChunk {
    pub payload: Box<ChunkPayload>,

    pub handle: Arc<ResRef>,

    pub level_deltas: [u32; 4],
}

impl TerrainChunk {
    const COORDINATE_BITS: u32 = 28;
    const COORDINATE_MASK: u64 = (1 << Self::COORDINATE_BITS) - 1;

    pub fn key(coordinate: ChunkCoordinate) -> u64 {
        ((coordinate.level as u64) << (Self::COORDINATE_BITS * 2))
            | ((coordinate.x as u32 as u64 & Self::COORDINATE_MASK) << Self::COORDINATE_BITS)
            | (coordinate.z as u32 as u64 & Self::COORDINATE_MASK)
    }
}
