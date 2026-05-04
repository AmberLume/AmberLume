use crate::input_handler::hardware_pointer_event::HardwarePointerEvent;
use crate::input_handler::hardware_key_codes::HardwareKeyCode;
use crate::input_handler::input_frame::{PointerId, InputFrame};

pub struct InputHandler {
    state: InputFrame,
}

impl InputHandler {
    pub fn create() -> Self {
        Self {
            state: InputFrame::create(),
        }
    }

    pub fn push_pointer_event(&mut self, id: &PointerId, event: HardwarePointerEvent) {
        match event {
            HardwarePointerEvent::Move { position } => {
                self.state.push_pointer_move(&id, position);
            }
            HardwarePointerEvent::Button { button, pressed } => {
                self.state.set_pointer_button(&id, button, pressed);
            }
            HardwarePointerEvent::Scroll { delta } => { 
                self.state.push_pointer_scroll(&id, delta);
            }
        };
    }

    pub fn push_keycode(&mut self, keycode: HardwareKeyCode, pressed: bool) {
        self.state.set_keycode(keycode, pressed)
    }

    pub fn pull(&mut self) -> InputFrame {
        let frame = self.state.clone();

        self.state.advance();

        frame
    }
}
