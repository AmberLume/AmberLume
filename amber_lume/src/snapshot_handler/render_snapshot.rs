use crate::physics::physics_debug_render::PhysicsDebugLine;
use crate::snapshot_handler::resolved_camera::ResolvedCamera;
use glam::{Mat4, Vec3};

pub struct RenderSnapshot {
    pub camera: ResolvedCamera,

    pub global_shadows_direction: Vec3,

    pub time: f32,

    pub entities: Vec<RenderEntity>,

    pub physics_debug_lines: Vec<PhysicsDebugLine>,
}

pub struct RenderEntity {
    pub transform_matrix: Mat4,

    pub mesh_id: u32,
    pub animation: Option<EntityAnimation>,
}

pub struct EntityAnimation {
    pub animation_id: u32,
    pub skeleton_id: u32,
    pub bone_transform_offset: u32,
    pub time: f32,

    pub previous_animation_id: u32,
    pub previous_time: f32,
    pub blend_factor: f32,
}
