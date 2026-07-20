use crate::hardware_keyboard::key_code::HardwareKeyCode;
use crate::signal::key_phase::KeyPhase;

#[derive(Clone, Debug)]
pub(crate) struct HardwareKeyboard {
    keys: [KeyPhase; HardwareKeyCode::Count as usize],
    consumed: [bool; HardwareKeyCode::Count as usize],
}

impl HardwareKeyboard {
    pub(crate) fn new() -> Self {
        Self {
            keys: [KeyPhase::Up; HardwareKeyCode::Count as usize],
            consumed: [false; HardwareKeyCode::Count as usize],
        }
    }

    pub(crate) fn set(&mut self, code: HardwareKeyCode, pressed: bool) {
        self.keys[code as usize].set(pressed);
    }

    pub(crate) fn advance(&mut self) {
        self.consumed = [false; HardwareKeyCode::Count as usize];

        for key in self.keys.iter_mut() {
            key.advance();
        }
    }

    pub(crate) fn resolve(&mut self, code: HardwareKeyCode, consume: bool) -> KeyPhase {
        let index = code as usize;

        if self.consumed[index] {
            return KeyPhase::Up;
        }

        if consume {
            self.consumed[index] = true;
        }

        self.keys[index]
    }
}
