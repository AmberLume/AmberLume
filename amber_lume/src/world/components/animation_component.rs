use crate::animation::animation_mapping::AnimationMapping;
use crate::animation::animation_state::AnimationState;
use shipyard::Component;
use std::sync::Arc;
use crate::animation::play_mode::PlayMode;

#[derive(Component)]
pub struct AnimationComponent<S: AnimationState + Send + Sync> {
    pub current_state: S,

    pub mapping: Arc<AnimationMapping>,
    pub time: f32,
    pub finished: bool,

    pub blend_from_state: S,
    pub blend_from_time: f32,
    pub blend_elapsed: f32,
    pub blend_duration: f32,
    pub blending: bool,

    pub last_state: S,
}

#[derive(Component)]
pub enum AnimationBlueprintComponent {
    Humanoid,
}

impl<S: AnimationState + Send + Sync> AnimationComponent<S> {
    pub fn can_interrupt(&self) -> bool {
        let entry = &self.mapping.entries[self.current_state.as_index() as usize];
        
        match &entry.mode {
            PlayMode::Loop => true,
            PlayMode::OnceCancellable { .. } => true,
            PlayMode::Once { .. } => self.finished,
        }
    }
}
