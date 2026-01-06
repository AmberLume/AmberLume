use crate::input_handler::keycodes::Keycode;

#[derive(Copy, Clone, Debug)]
pub enum KeyEvent {
    Pressed(Keycode),
    Released(Keycode),
}
