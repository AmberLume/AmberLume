use animation::playback::animation_playback::AnimationPlayback;
use animation::state_machine::animation_state_machine::AnimationStateMachine;
use shipyard::Component;
use std::sync::Arc;

#[derive(Component)]
pub struct AnimationComponent {
    pub state_machine: Arc<AnimationStateMachine>,
    pub playback: AnimationPlayback,
    pub previous_playback: AnimationPlayback,
}

impl AnimationComponent {
    pub fn create(state_machine: Arc<AnimationStateMachine>) -> Self {
        let playback = AnimationPlayback::create(&state_machine);

        Self {
            state_machine,
            playback,
            previous_playback: playback,
        }
    }
}
