#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KeyPhase {
    Up,
    JustPressed,
    Held,
    JustReleased,
}

impl KeyPhase {
    pub fn is_down(self) -> bool {
        matches!(self, KeyPhase::JustPressed | KeyPhase::Held)
    }

    pub fn is_just_pressed(self) -> bool {
        self == KeyPhase::JustPressed
    }

    pub fn is_just_released(self) -> bool {
        self == KeyPhase::JustReleased
    }

    pub(crate) fn set(&mut self, pressed: bool) {
        *self = match (*self, pressed) {
            (KeyPhase::Up | KeyPhase::JustReleased, true) => KeyPhase::JustPressed,
            (KeyPhase::Held | KeyPhase::JustPressed, false) => KeyPhase::JustReleased,
            _ => *self,
        };
    }

    pub(crate) fn advance(&mut self) {
        *self = match *self {
            KeyPhase::JustPressed => KeyPhase::Held,
            KeyPhase::JustReleased => KeyPhase::Up,
            other => other,
        };
    }
}
