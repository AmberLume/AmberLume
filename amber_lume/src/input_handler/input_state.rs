use crate::input_handler::keycodes::Keycode;

#[derive(Copy, Clone, Debug)]
pub struct InputState {
    keys: [bool; Keycode::Count as usize],
}

impl InputState {
    pub fn create() -> Self {
        Self {
            keys: [false; Keycode::Count as usize],
        }
    }

    pub fn set(&mut self, key: Keycode, pressed: bool) {
        self.keys[key as usize] = pressed;
    }

    pub fn is_down(&self, key: Keycode) -> bool {
        self.keys[key as usize]
    }
}
