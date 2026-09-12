use crate::parameters::animation_parameters::AnimationParameters;
use crate::state_machine::animation_state::AnimationState;
use crate::state_machine::play_mode::PlayMode;
use crate::transition::animation_transition::AnimationTransition;

pub struct AnimationStateMachine {
    pub states: Vec<AnimationState>,
    pub transitions: Vec<AnimationTransition>,

    pub initial_state: u16,
}

impl AnimationStateMachine {
    pub fn new(
        states: Vec<AnimationState>,
        transitions: Vec<AnimationTransition>,
        initial_state: u16,
    ) -> Self {
        let count = states.len() as u16;

        assert!(
            initial_state < count,
            "AnimationStateMachine: initial state {} is out of {} states",
            initial_state,
            count,
        );

        for transition in &transitions {
            assert!(
                transition.target < count,
                "AnimationStateMachine: transition target {} is out of {} states",
                transition.target,
                count,
            );
        }

        Self {
            states,
            transitions,

            initial_state,
        }
    }

    pub fn transition(
        &self,
        state: u16,
        finished: bool,
        parameters: &AnimationParameters,
    ) -> Option<&AnimationTransition> {
        if matches!(self.states[state as usize].mode, PlayMode::Once) && !finished {
            return None;
        }

        self.transitions.iter().find(|transition| {
            transition.target != state
                && transition.source.matches(state)
                && transition
                    .conditions
                    .iter()
                    .all(|condition| condition.holds(parameters, finished))
        })
    }
}
