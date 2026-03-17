use crate::physics::physics_debug_render::PhysicsDebugLine;
use crate::world::unique::world_camera_unique::CameraStamp;
use glam::{Mat4, Vec3};

pub struct WorldSnapshot {
    pub camera_stamp: CameraStamp,
    pub global_shadows_direction: Vec3,

    pub entities: Vec<WorldEntity>,

    pub physics_debug_lines: Vec<PhysicsDebugLine>,
}

impl WorldSnapshot {
    pub fn default() -> Self {
        Self {
            camera_stamp: CameraStamp::default(),
            global_shadows_direction: Vec3::NEG_Y,

            entities: Vec::new(),

            physics_debug_lines: Vec::new(),
        }
    }
}

pub struct WorldEntity {
    pub transform_matrix: Mat4,

    pub model_id: u32,
}
