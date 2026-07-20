use crate::hardware_pointer::key_codes::HardwarePointerKeyCodes;
use crate::Point;

pub enum HardwarePointerEvent {
    Move {
        position: Point,
    },
    Motion {
        delta: Point,
    },
    Button {
        button: HardwarePointerKeyCodes,
        pressed: bool,
    },
    Scroll {
        delta: Point,
    },
}
