use crate::chunk::ChunkCoordinate;

pub struct ResidencyUpdate {
    pub requested: Vec<ChunkCoordinate>,
    pub retired: Vec<ChunkCoordinate>,
}
