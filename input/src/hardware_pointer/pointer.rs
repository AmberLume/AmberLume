use crate::hardware_pointer::key_codes::HardwarePointerKeyCodes;
use crate::hardware_pointer::point::Point;
use crate::signal::key_phase::KeyPhase;

#[derive(Clone, Debug)]
pub(crate) struct HardwarePointer {
    pub(crate) position: Option<Point>,
    pub(crate) motion: Option<Point>,
    pub(crate) scroll: Option<Point>,

    buttons: [KeyPhase; HardwarePointerKeyCodes::Count as usize],
    consumed: [bool; HardwarePointerKeyCodes::Count as usize],
}

impl HardwarePointer {
    pub(crate) fn new() -> Self {
        Self {
            position: None,
            motion: None,
            scroll: None,
            buttons: [KeyPhase::Up; HardwarePointerKeyCodes::Count as usize],
            consumed: [false; HardwarePointerKeyCodes::Count as usize],
        }
    }

    pub(crate) fn set_button(&mut self, button: HardwarePointerKeyCodes, pressed: bool) {
        self.buttons[button as usize].set(pressed);
    }

    pub(crate) fn advance(&mut self) {
        self.position = None;
        self.motion = None;
        self.scroll = None;
        self.consumed = [false; HardwarePointerKeyCodes::Count as usize];

        for button in self.buttons.iter_mut() {
            button.advance();
        }
    }

    pub(crate) fn resolve_button(&mut self, button: HardwarePointerKeyCodes, consume: bool) -> KeyPhase {
        let index = button as usize;

        if self.consumed[index] {
            return KeyPhase::Up;
        }

        if consume {
            self.consumed[index] = true;
        }

        self.buttons[index]
    }
}
