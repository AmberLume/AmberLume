use std::collections::HashMap;
use crate::hardware_keyboard::key_code::HardwareKeyCode;
use crate::hardware_keyboard::keyboard::HardwareKeyboard;
use crate::hardware_pointer::event::HardwarePointerEvent;
use crate::hardware_pointer::key_codes::HardwarePointerKeyCodes;
use crate::hardware_pointer::point::Point;
use crate::hardware_pointer::pointer::HardwarePointer;
use crate::hardware_pointer::pointer_id::PointerId;
use crate::signal::key_phase::KeyPhase;

pub struct InputHandler {
    keyboard: HardwareKeyboard,
    pointers: HashMap<PointerId, HardwarePointer>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            keyboard: HardwareKeyboard::new(),
            pointers: HashMap::new(),
        }
    }

    pub fn push_key(&mut self, keycode: HardwareKeyCode, pressed: bool) {
        self.keyboard.set(keycode, pressed);
    }

    pub fn push_pointer(&mut self, id: PointerId, event: HardwarePointerEvent) {
        let pointer = self.pointers.entry(id).or_insert_with(HardwarePointer::new);

        match event {
            HardwarePointerEvent::Move { position } => {
                pointer.position = Some(position);
            }
            HardwarePointerEvent::Motion { delta } => {
                pointer.motion.get_or_insert(Point::ZERO).add(delta);
            }
            HardwarePointerEvent::Scroll { delta } => {
                pointer.scroll.get_or_insert(Point::ZERO).add(delta);
            }
            HardwarePointerEvent::Button { button, pressed } => {
                pointer.set_button(button, pressed);
            }
        }
    }

    pub fn advance(&mut self) {
        self.keyboard.advance();

        for pointer in self.pointers.values_mut() {
            pointer.advance();
        }
    }

    pub fn key(&mut self, code: HardwareKeyCode, consume: bool) -> KeyPhase {
        self.keyboard.resolve(code, consume)
    }

    pub fn button(&mut self, button: HardwarePointerKeyCodes, consume: bool) -> KeyPhase {
        self.primary_pointer().resolve_button(button, consume)
    }

    pub fn position(&mut self, consume: bool) -> Option<Point> {
        let pointer = self.primary_pointer();
        let position = pointer.position;

        if consume {
            pointer.position = None;
        }

        position
    }

    pub fn motion(&mut self, consume: bool) -> Option<Point> {
        let pointer = self.primary_pointer();
        let motion = pointer.motion;

        if consume {
            pointer.motion = None;
        }

        motion
    }

    pub fn scroll(&mut self, consume: bool) -> Option<Point> {
        let pointer = self.primary_pointer();
        let scroll = pointer.scroll;

        if consume {
            pointer.scroll = None;
        }

        scroll
    }

    fn primary_pointer(&mut self) -> &mut HardwarePointer {
        self.pointers.entry(PointerId::new(0)).or_insert_with(HardwarePointer::new)
    }
}
