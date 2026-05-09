use crate::input_handler::hardware_pointer_key_codes::HardwarePointerKeyCodes;

pub enum HardwarePointerEvent {
    Move {
        position: (f32, f32),
    },
    Motion {
        delta: (f32, f32),
    },
    Button {
        button: HardwarePointerKeyCodes,
        pressed: bool,
    },
    Scroll {
        delta: (f32, f32),
    },
}
