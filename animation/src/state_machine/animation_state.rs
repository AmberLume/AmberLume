use crate::state_machine::play_mode::PlayMode;
use resource_residency::ResRef;
use std::sync::Arc;

pub struct AnimationState {
    pub clip: Arc<ResRef>,

    pub duration: f32,
    pub speed: f32,

    pub mode: PlayMode,
}
