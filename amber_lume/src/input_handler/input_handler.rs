use std::mem::swap;
use crate::input_handler::input_event::KeyEvent;
use crate::input_handler::input_state::InputState;

pub struct InputHandler {
    state: InputState,

    input_events: Vec<KeyEvent>,
    process_events: Vec<KeyEvent>,
}

impl InputHandler {
    pub fn create() -> Self {
        Self {
            state: InputState::create(),

            input_events: Vec::with_capacity(10),
            process_events: Vec::with_capacity(10),
        }
    }

    pub fn push(&mut self, key_event: KeyEvent) {
        match key_event {
            KeyEvent::Pressed(key) => self.state.set(key, true),
            KeyEvent::Released(key) => self.state.set(key, false),
        }

        self.input_events.push(key_event);
    }

    pub fn pull(&mut self) -> (InputState, &[KeyEvent]) {
        self.process_events.clear();
        swap(&mut self.input_events, &mut self.process_events);

        (self.state, &self.process_events)
    }

    pub fn get_state(&self) -> InputState {
        self.state.clone()
    }
}
