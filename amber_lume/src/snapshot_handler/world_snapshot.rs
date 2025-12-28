use glam::Mat4;
use crate::world::unique::world_camera_unique::CameraStamp;

pub struct WorldSnapshot {
    pub camera_stamp: CameraStamp,

    pub entities: Vec<WorldEntity>,
}

impl WorldSnapshot {
    pub fn default() -> Self {
        Self {
            camera_stamp: CameraStamp::default(),
            entities: Vec::new(),
        }
    }
}

pub struct WorldEntity {
    pub transform_matrix: Mat4,

    pub model_id: u32,
}
