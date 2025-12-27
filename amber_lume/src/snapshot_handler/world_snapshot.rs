use glam::Mat4;

pub struct WorldSnapshot {
    pub camera_projection_matrix: Mat4,

    pub entities: Vec<WorldEntity>,
}

impl WorldSnapshot {
    pub fn default() -> Self {
        Self {
            camera_projection_matrix: Mat4::IDENTITY,
            entities: Vec::new(),
        }
    }
}

pub struct WorldEntity {
    pub transform_matrix: Mat4,

    pub mesh_id: u32,
}
