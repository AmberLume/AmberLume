use shipyard::Component;
use terrain::ChunkCoordinate;

#[derive(Component)]
pub struct TerrainChunkComponent {
    pub coordinate: ChunkCoordinate,

    pub vertex_offset: u32,
    pub level_deltas: [u32; 4],
}
