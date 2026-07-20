use std::mem::take;
use parking_lot::Mutex;
use amber_lume::input::HardwareKeyCode;

#[derive(Default)]
pub struct InputHandler {
    keycode_states: Mutex<Vec<(HardwareKeyCode, bool)>>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&self, key_code: HardwareKeyCode, state: bool) {
        self.keycode_states
            .lock()
            .push((key_code, state));
    }

    pub fn drain(&self) -> Vec<(HardwareKeyCode, bool)> {
        take(&mut *self.keycode_states.lock())
    }
}