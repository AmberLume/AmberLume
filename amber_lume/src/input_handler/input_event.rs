use crate::input_handler::hardware_key_codes::HardwareKeyCode;

#[derive(Copy, Clone, Debug)]
pub enum KeyEvent {
    Pressed(HardwareKeyCode),
    Released(HardwareKeyCode),
}
