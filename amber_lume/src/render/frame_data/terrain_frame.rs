use crate::render::frame_data::terrain_chunk_view::TerrainChunkView;
use crate::render::frame_data::terrain_generate_request::TerrainGenerateRequest;
use crate::render::frame_data::terrain_stitch_request::TerrainStitchRequest;

pub struct TerrainFrame {
    pub generate_requests: Vec<TerrainGenerateRequest>,
    pub stitch_requests: Vec<TerrainStitchRequest>,

    pub chunks: Vec<TerrainChunkView>,
}
