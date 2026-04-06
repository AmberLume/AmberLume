use crate::animation::animation_mapping::AnimationMapping;
use crate::animation::animation_state::AnimationState;
use shipyard::Component;
use std::sync::Arc;

#[derive(Component)]
pub struct AnimationComponent<S: AnimationState + Send + Sync> {
    pub current_state: S,

    pub mapping: Arc<AnimationMapping>,
    pub time: f32,
    pub previous_state_index: u16,
    pub finished: bool,
}

#[derive(Component)]
pub enum AnimationBlueprintComponent {
    Humanoid,
}
