use android_activity::input::{InputEvent, KeyAction, Keycode as AKeycode, MotionAction};
use input::{HardwareKeyCode, HardwarePointerEvent, HardwarePointerKeyCodes, Point, PointerId};
use crate::EngineEvent;

pub fn translate_input_event(event: &InputEvent) -> Vec<EngineEvent> {
    match event {
        InputEvent::MotionEvent(motion) => {
            let pointer_index = motion.pointer_index();
            let pointer = motion.pointer_at_index(pointer_index);

            if pointer.pointer_id() != 0 {
                return Vec::new();
            }

            let position = Point::new(pointer.x(), pointer.y());
            let id = PointerId::new(pointer_index as i32);

            match motion.action() {
                MotionAction::Down => vec![
                    EngineEvent::Pointer { id, event: HardwarePointerEvent::Move { position } },
                    EngineEvent::Pointer {
                        id,
                        event: HardwarePointerEvent::Button {
                            button: HardwarePointerKeyCodes::Left,
                            pressed: true,
                        },
                    },
                ],
                MotionAction::Move => vec![
                    EngineEvent::Pointer { id, event: HardwarePointerEvent::Move { position } },
                ],
                MotionAction::Up => vec![
                    EngineEvent::Pointer { id, event: HardwarePointerEvent::Move { position } },
                    EngineEvent::Pointer {
                        id,
                        event: HardwarePointerEvent::Button {
                            button: HardwarePointerKeyCodes::Left,
                            pressed: false,
                        },
                    },
                ],
                MotionAction::Cancel => vec![
                    EngineEvent::Pointer {
                        id,
                        event: HardwarePointerEvent::Button {
                            button: HardwarePointerKeyCodes::Left,
                            pressed: false,
                        },
                    },
                ],
                _ => Vec::new(),
            }
        }
        InputEvent::KeyEvent(key) => {
            let Some(code) = android_keycode_to_hardware_keycode(key.key_code()) else {
                return Vec::new();
            };

            if key.repeat_count() != 0 {
                return Vec::new();
            }

            let pressed = match key.action() {
                KeyAction::Down => true,
                KeyAction::Up => false,
                _ => return Vec::new(),
            };

            vec![EngineEvent::Keycode { code, pressed }]
        }
        _ => Vec::new(),
    }
}

fn android_keycode_to_hardware_keycode(code: AKeycode) -> Option<HardwareKeyCode> {
    Some(match code {
        AKeycode::Escape => HardwareKeyCode::Esc,
        AKeycode::F1 => HardwareKeyCode::F1,
        AKeycode::F2 => HardwareKeyCode::F2,
        AKeycode::F3 => HardwareKeyCode::F3,
        AKeycode::F4 => HardwareKeyCode::F4,
        AKeycode::F5 => HardwareKeyCode::F5,
        AKeycode::F6 => HardwareKeyCode::F6,
        AKeycode::F7 => HardwareKeyCode::F7,
        AKeycode::F8 => HardwareKeyCode::F8,
        AKeycode::F9 => HardwareKeyCode::F9,
        AKeycode::F10 => HardwareKeyCode::F10,
        AKeycode::F11 => HardwareKeyCode::F11,
        AKeycode::F12 => HardwareKeyCode::F12,
        AKeycode::Q => HardwareKeyCode::Q,
        AKeycode::W => HardwareKeyCode::W,
        AKeycode::E => HardwareKeyCode::E,
        AKeycode::R => HardwareKeyCode::R,
        AKeycode::T => HardwareKeyCode::T,
        AKeycode::Y => HardwareKeyCode::Y,
        AKeycode::U => HardwareKeyCode::U,
        AKeycode::I => HardwareKeyCode::I,
        AKeycode::O => HardwareKeyCode::O,
        AKeycode::P => HardwareKeyCode::P,
        AKeycode::A => HardwareKeyCode::A,
        AKeycode::S => HardwareKeyCode::S,
        AKeycode::D => HardwareKeyCode::D,
        AKeycode::F => HardwareKeyCode::F,
        AKeycode::G => HardwareKeyCode::G,
        AKeycode::H => HardwareKeyCode::H,
        AKeycode::J => HardwareKeyCode::J,
        AKeycode::K => HardwareKeyCode::K,
        AKeycode::L => HardwareKeyCode::L,
        AKeycode::Z => HardwareKeyCode::Z,
        AKeycode::X => HardwareKeyCode::X,
        AKeycode::C => HardwareKeyCode::C,
        AKeycode::V => HardwareKeyCode::V,
        AKeycode::B => HardwareKeyCode::B,
        AKeycode::N => HardwareKeyCode::N,
        AKeycode::M => HardwareKeyCode::M,
        AKeycode::DpadUp => HardwareKeyCode::ArrowUp,
        AKeycode::DpadDown => HardwareKeyCode::ArrowDown,
        AKeycode::DpadLeft => HardwareKeyCode::ArrowLeft,
        AKeycode::DpadRight => HardwareKeyCode::ArrowRight,
        AKeycode::Space => HardwareKeyCode::Space,
        _ => return None,
    })
}
