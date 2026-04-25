use amber_lume::input_handler::input_event::KeyEvent as LumeKeyEvent;
use amber_lume::input_handler::keycodes::Keycode;
use amber_lume::ui::events::ui_events::{EventState, MouseButton, MouseEvent};
use android_activity::input::{InputEvent, KeyAction, Keycode as AKeycode, MotionAction, };
use core::lume::Lume;

pub fn handle_input_event(event: &InputEvent, lume: &mut Lume) {
    match event {
        InputEvent::MotionEvent(motion) => {
            let pointer_index = motion.pointer_index();
            let pointer = motion.pointer_at_index(pointer_index);

            if pointer.pointer_id() != 0 {
                return;
            }

            let position = [pointer.x(), pointer.y()];

            match motion.action() {
                MotionAction::Down => {
                    lume.on_mouse_event(MouseEvent::Move { position });
                    lume.on_mouse_event(MouseEvent::Click {
                        button: MouseButton::Left,
                        state: EventState::Down,
                    });
                }
                MotionAction::Move => {
                    lume.on_mouse_event(MouseEvent::Move { position });
                }
                MotionAction::Up => {
                    lume.on_mouse_event(MouseEvent::Move { position });
                    lume.on_mouse_event(MouseEvent::Click {
                        button: MouseButton::Left,
                        state: EventState::Up,
                    });
                }
                MotionAction::Cancel => {
                    lume.on_mouse_event(MouseEvent::Click {
                        button: MouseButton::Left,
                        state: EventState::Up,
                    });
                }
                _ => {}
            }
        }
        InputEvent::KeyEvent(key) => {
            let Some(keycode) = android_keycode_to_lume(key.key_code()) else {
                return;
            };

            if key.repeat_count() != 0 {
                return;
            }

            let lume_event = match key.action() {
                KeyAction::Down => LumeKeyEvent::Pressed(keycode),
                KeyAction::Up => LumeKeyEvent::Released(keycode),
                _ => return,
            };

            lume.on_key_event(lume_event);
        }
        _ => {}
    }
}

fn android_keycode_to_lume(code: AKeycode) -> Option<Keycode> {
    Some(match code {
        AKeycode::Escape => Keycode::Esc,
        AKeycode::DpadUp => Keycode::ArrowUp,
        AKeycode::DpadDown => Keycode::ArrowDown,
        AKeycode::DpadLeft => Keycode::ArrowLeft,
        AKeycode::DpadRight => Keycode::ArrowRight,
        AKeycode::Space => Keycode::Space,
        AKeycode::F1 => Keycode::F1,
        AKeycode::F2 => Keycode::F2,
        AKeycode::F3 => Keycode::F3,
        AKeycode::F4 => Keycode::F4,
        AKeycode::F5 => Keycode::F5,
        AKeycode::F6 => Keycode::F6,
        AKeycode::F7 => Keycode::F7,
        AKeycode::F8 => Keycode::F8,
        AKeycode::F9 => Keycode::F9,
        AKeycode::F10 => Keycode::F10,
        AKeycode::F11 => Keycode::F11,
        AKeycode::F12 => Keycode::F12,
        AKeycode::Q => Keycode::Q,
        AKeycode::W => Keycode::W,
        AKeycode::E => Keycode::E,
        AKeycode::R => Keycode::R,
        AKeycode::A => Keycode::A,
        AKeycode::S => Keycode::S,
        AKeycode::D => Keycode::D,
        AKeycode::F => Keycode::F,
        AKeycode::Z => Keycode::Z,
        AKeycode::X => Keycode::X,
        AKeycode::C => Keycode::C,
        AKeycode::V => Keycode::V,
        _ => return None,
    })
}
