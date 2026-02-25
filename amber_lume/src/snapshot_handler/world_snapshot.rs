use glam::{Mat4, Vec3, Vec4};
use crate::world::unique::world_camera_unique::CameraStamp;

pub struct WorldSnapshot {
    pub camera_stamp: CameraStamp,
    pub global_shadows_direction: Vec3,

    pub entities: Vec<WorldEntity>,

    pub colliders: Vec<WorldCollider>,
}

impl WorldSnapshot {
    pub fn default() -> Self {
        Self {
            camera_stamp: CameraStamp::default(),
            global_shadows_direction: Vec3::NEG_Y,

            entities: Vec::new(),

            colliders: Vec::new(),
        }
    }
}

pub struct WorldEntity {
    pub transform_matrix: Mat4,

    pub model_id: u32,
}

pub struct WorldCollider {
    pub transform_matrix: Mat4,
    pub half_extents: Vec4,
    pub shape_type: u32,
    pub color: Vec4,
}
