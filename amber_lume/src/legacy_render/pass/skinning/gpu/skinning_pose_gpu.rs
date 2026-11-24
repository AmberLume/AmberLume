use bytemuck::{Pod, Zeroable};
use render_snapshot::AnimationPose;

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct SkinningPoseGPU {
    pub animation_id: u32,
    pub time: f32,

    pub blend_from_animation_id: u32,
    pub blend_from_time: f32,
    pub blend_factor: f32,
}

impl SkinningPoseGPU {
    pub fn create(pose: &AnimationPose) -> Self {
        Self {
            animation_id: pose.animation_id,
            time: pose.time,

            blend_from_animation_id: pose.blend_from_animation_id,
            blend_from_time: pose.blend_from_time,
            blend_factor: pose.blend_factor,
        }
    }
}
