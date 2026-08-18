use crate::chunk::{ChunkCoordinate, ChunkPayload};
use anyhow::Result;

pub trait TerrainSource: Send + Sync {
    fn fill(&self, coordinate: ChunkCoordinate, heights: &mut [f32]) -> Result<()>;

    fn load_into(&self, coordinate: ChunkCoordinate, payload: &mut ChunkPayload) -> Result<()> {
        self.fill(coordinate, payload.heights_mut())?;

        payload.recalculate_bounds();

        Ok(())
    }

    fn load(&self, coordinate: ChunkCoordinate) -> Result<ChunkPayload> {
        let mut payload = ChunkPayload::empty(coordinate);

        self.load_into(coordinate, &mut payload)?;

        Ok(payload)
    }
}
