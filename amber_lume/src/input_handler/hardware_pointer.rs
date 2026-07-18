use crate::input_handler::hardware_pointer_key_codes::HardwarePointerKeyCodes;
use crate::input_handler::hardware_key_state::HardwareKeyState;

#[derive(Clone, Debug)]
pub struct HardwarePointer {
    pub position: Option<(f32, f32)>,
    pub position_delta: (f32, f32),
    pub scroll_delta: (f32, f32),
    pub buttons: [HardwareKeyState; HardwarePointerKeyCodes::Count as usize],
}

impl Default for HardwarePointer {
    fn default() -> Self {
        Self {
            position: None,
            position_delta: (0.0, 0.0),
            scroll_delta: (0.0, 0.0),
            buttons: [HardwareKeyState::default(); HardwarePointerKeyCodes::Count as usize],
        }
    }
}

impl HardwarePointer {
    pub fn set_button(&mut self, button: HardwarePointerKeyCodes, pressed: bool) {
        let current = self.buttons[button as usize];

        self.buttons[button as usize] = match (current, pressed) {
            (HardwareKeyState::Up | HardwareKeyState::JustReleased, true) => HardwareKeyState::JustPressed,
            (HardwareKeyState::Held | HardwareKeyState::JustPressed, false) => HardwareKeyState::JustReleased,
            _ => current,
        }
    }

    pub fn key_just_pressed(&self, button: HardwarePointerKeyCodes) -> bool {
        self.buttons[button as usize] == HardwareKeyState::JustPressed
    }

    pub fn key_pressed(&self, button: HardwarePointerKeyCodes) -> bool {
        let state = self.buttons[button as usize];

        state == HardwareKeyState::JustPressed || state == HardwareKeyState::Held
    }
}
