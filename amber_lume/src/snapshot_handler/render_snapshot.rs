use physics::PhysicsDebugLine;
use crate::snapshot_handler::resolved_camera::ResolvedCamera;
use glam::{Mat4, Vec3};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct RenderEntityId(pub u64);

pub struct RenderSnapshot {
    pub camera: ResolvedCamera,

    pub global_shadows_direction: Vec3,
    pub global_shadows_color: Vec3,
    pub global_shadows_intensity: f32,
    pub global_ibl_intensity: f32,

    pub time: f32,

    pub entities: Vec<RenderEntity>,

    pub physics_debug_lines: Vec<PhysicsDebugLine>,
}

pub struct RenderEntity {
    pub id: RenderEntityId,

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
