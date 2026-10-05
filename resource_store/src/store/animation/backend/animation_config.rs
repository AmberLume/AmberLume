use gpu_data::AnimationFrameGPU;
use resource_residency::ResRef;
use std::sync::Arc;

#[derive(Clone)]
pub struct AnimationConfig {
    pub name: String,
    pub skeleton: Arc<ResRef>,

    pub duration: f32,
    pub fps: f32,
    pub bone_count: u32,
    pub frame_count: u32,

    pub frames: Vec<AnimationFrameGPU>,
}
