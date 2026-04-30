use std::mem::take;
use std::sync::Mutex;
use amber_lume::input_handler::hardware_key_codes::HardwareKeyCode;

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
            .unwrap_or_else(|e| e.into_inner())
            .push((key_code, state));
    }

    pub fn drain(&self) -> Vec<(HardwareKeyCode, bool)> {
        take(&mut *self.keycode_states.lock().unwrap_or_else(|e| e.into_inner()))
    }
}