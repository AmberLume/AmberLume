use crate::parameters::animation_parameters::AnimationParameters;
use crate::state_machine::animation_state_machine::AnimationStateMachine;
use crate::state_machine::play_mode::PlayMode;

#[derive(Clone, Copy)]
pub struct AnimationPlayback {
    pub current_state: u16,
    pub time: f32,
    pub finished: bool,

    pub blend_from_state: u16,
    pub blend_from_time: f32,
    pub blend_elapsed: f32,
    pub blend_duration: f32,
}

impl AnimationPlayback {
    pub fn create(state_machine: &AnimationStateMachine) -> Self {
        Self {
            current_state: state_machine.initial_state,
            time: 0.0,
            finished: false,

            blend_from_state: state_machine.initial_state,
            blend_from_time: 0.0,
            blend_elapsed: 0.0,
            blend_duration: 0.0,
        }
    }

    pub fn blend_factor(&self) -> f32 {
        if self.blend_elapsed < self.blend_duration {
            self.blend_elapsed / self.blend_duration
        } else {
            1.0
        }
    }

    pub fn advance(
        &mut self,
        state_machine: &AnimationStateMachine,
        parameters: &AnimationParameters,
        delta: f32,
    ) {
        let state = &state_machine.states[self.current_state as usize];

        if !self.finished {
            self.time += delta * state.speed;

            match state.mode {
                PlayMode::Loop => {
                    if state.duration > 0.0 {
                        self.time %= state.duration;
                    }
                }
                PlayMode::Once | PlayMode::OnceCancellable => {
                    if self.time >= state.duration {
                        self.time = state.duration;
                        self.finished = true;
                    }
                }
            }
        }

        if let Some(transition) =
            state_machine.transition(self.current_state, self.finished, parameters)
        {
            self.blend_from_state = self.current_state;
            self.blend_from_time = self.time;
            self.blend_elapsed = 0.0;
            self.blend_duration = transition.blend_duration;

            self.current_state = transition.target;
            self.time = 0.0;
            self.finished = false;
        }

        if self.blend_elapsed < self.blend_duration {
            self.blend_elapsed += delta;

            let previous_state = &state_machine.states[self.blend_from_state as usize];

            match previous_state.mode {
                PlayMode::Loop => {
                    self.blend_from_time += delta * previous_state.speed;

                    if previous_state.duration > 0.0 {
                        self.blend_from_time %= previous_state.duration;
                    }
                }
                PlayMode::Once | PlayMode::OnceCancellable => {
                    self.blend_from_time = self.blend_from_time.min(previous_state.duration);
                }
            }
        }
    }
}
