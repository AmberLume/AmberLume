use animation::playback::animation_playback::AnimationPlayback;
use animation::state_machine::animation_state_machine::AnimationStateMachine;
use resource_residency::ResRef;
use shipyard::Component;
use std::sync::Arc;

#[derive(Component)]
pub struct AnimationComponent {
    pub state_machine: Arc<AnimationStateMachine>,
    pub playback: AnimationPlayback,

    pub skeleton: Arc<ResRef>,
}

impl AnimationComponent {
    pub fn create(state_machine: Arc<AnimationStateMachine>, skeleton: Arc<ResRef>) -> Self {
        let playback = AnimationPlayback::create(&state_machine);

        Self {
            state_machine,
            playback,

            skeleton,
        }
    }
}
