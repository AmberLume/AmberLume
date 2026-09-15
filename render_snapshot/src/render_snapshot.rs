use crate::camera_view::CameraView;
use crate::animation_pose::AnimationPose;
use crate::debug_line::DebugLine;
use glam::{Mat4, Vec3};
use resource_store::GeometryChanges;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct RenderEntityId(pub u64);

impl RenderEntityId {
    pub const STATIC: Self = Self(0);
}

pub struct RenderSnapshot {
    pub camera: CameraView,

    pub global_shadows_direction: Vec3,
    pub global_shadows_color: Vec3,
    pub global_shadows_intensity: f32,
    pub global_ibl_intensity: f32,

    pub time: f32,

    pub entities: Vec<RenderEntity>,

    pub geometry_changes: GeometryChanges,

    pub debug_lines: Vec<DebugLine>,
}

pub struct RenderEntity {
    pub id: RenderEntityId,

    pub transform_matrix: Mat4,

    pub mesh_id: u32,
    pub animation: Option<EntityAnimation>,
    pub outline: [f32; 4],
}

pub struct EntityAnimation {
    pub skeleton_id: u32,

    pub pose: AnimationPose,
    pub previous_pose: AnimationPose,
}
