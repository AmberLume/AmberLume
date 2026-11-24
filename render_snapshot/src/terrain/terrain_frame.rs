use crate::terrain::terrain_chunk_view::TerrainChunkView;
use crate::terrain::terrain_generate_request::TerrainGenerateRequest;
use crate::terrain::terrain_stitch_request::TerrainStitchRequest;

pub struct TerrainFrame {
    pub generate_requests: Vec<TerrainGenerateRequest>,
    pub stitch_requests: Vec<TerrainStitchRequest>,

    pub chunks: Vec<TerrainChunkView>,
}
