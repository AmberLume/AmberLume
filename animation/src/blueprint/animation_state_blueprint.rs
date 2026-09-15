use crate::state_machine::play_mode::PlayMode;
use resource_data::resource_handle::AnimationResource;

pub struct AnimationStateBlueprint {
    pub clip: AnimationResource,
    pub speed: f32,
    pub mode: PlayMode,
}
