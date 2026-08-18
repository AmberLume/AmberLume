use crate::render::frame_data::terrain_generate_request::TerrainGenerateRequest;

pub struct TerrainFrame {
    pub generate_requests: Vec<TerrainGenerateRequest>,
}
