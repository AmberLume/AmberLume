use std::collections::hash_map::Entry;
use std::collections::HashMap;
use crate::input_handler::hardware_pointer::HardwarePointer;
use crate::input_handler::hardware_pointer_key_codes::HardwarePointerKeyCodes;
use crate::input_handler::hardware_key_codes::HardwareKeyCode;
use crate::input_handler::hardware_key_state::HardwareKeyState;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct PointerId {
    pub id: i32,
}

impl PointerId {
    pub fn new(id: i32) -> Self {
        Self { id }
    }
}

#[derive(Clone, Debug)]
pub struct InputFrame {
    keys: [HardwareKeyState; HardwareKeyCode::Count as usize],
    pointers: HashMap<PointerId, HardwarePointer>,
}

impl InputFrame {
    pub fn create() -> Self {
        Self {
            keys: [HardwareKeyState::default(); HardwareKeyCode::Count as usize],
            pointers: HashMap::new(),
        }
    }

    pub fn advance(&mut self) {
        for hardware_key_state in self.keys.iter_mut() {
            *hardware_key_state = match *hardware_key_state {
                HardwareKeyState::JustPressed => HardwareKeyState::Held,
                HardwareKeyState::JustReleased => HardwareKeyState::Up,
                other => other,
            }
        }

        for (_, pointer) in self.pointers.iter_mut() {
            pointer.scroll_delta = (0.0, 0.0);
            pointer.position_delta = (0.0, 0.0);

            for pointer_button in pointer.buttons.iter_mut() {
                *pointer_button = match *pointer_button {
                    HardwareKeyState::JustPressed => HardwareKeyState::Held,
                    HardwareKeyState::JustReleased => HardwareKeyState::Up,
                    other => other,
                }
            }
        }
    }

    pub fn set_keycode(&mut self, keycode: HardwareKeyCode, pressed: bool) {
        let current = self.keys[keycode as usize];

        self.keys[keycode as usize] = match (current, pressed) {
            (HardwareKeyState::Up | HardwareKeyState::JustReleased, true) => HardwareKeyState::JustPressed,
            (HardwareKeyState::Held | HardwareKeyState::JustPressed, false) => HardwareKeyState::JustReleased,
            _ => current,
        }
    }

    pub fn push_pointer_move(&mut self, id: &PointerId, position: (f32, f32)) {
        match self.pointers.entry(*id) {
            Entry::Occupied(mut entry) => {
                let pointer = entry.get_mut();

                pointer.position_delta = (position.0 - pointer.position.0, position.1 - pointer.position.1);
                pointer.position = position;
            }
            Entry::Vacant(e) => {
                e.insert(HardwarePointer::new(position));
            }
        }
    }

    pub fn push_pointer_scroll(&mut self, id: &PointerId, scroll_delta: (f32, f32)) {
        if let Some(pointer) = self.pointers.get_mut(&id) {
            pointer.scroll_delta.0 += scroll_delta.0;
            pointer.scroll_delta.1 += scroll_delta.1;
        }
    }

    pub fn set_pointer_button(&mut self, id: &PointerId, button: HardwarePointerKeyCodes, pressed: bool) {
        if let Some(pointer) = self.pointers.get_mut(&id) {
            pointer.set_button(button, pressed);
        }
    }

    pub fn push_cursor_removed(&mut self, id: PointerId) {
        self.pointers.remove(&id);
    }

    pub fn is_down(&self, keycode: HardwareKeyCode) -> bool {
        matches!(self.keys[keycode as usize], HardwareKeyState::JustPressed | HardwareKeyState::Held)
    }

    pub fn just_pressed(&self, keycode: HardwareKeyCode) -> bool {
        self.keys[keycode as usize] == HardwareKeyState::JustPressed
    }

    pub fn just_released(&self, keycode: HardwareKeyCode) -> bool {
        self.keys[keycode as usize] == HardwareKeyState::JustReleased
    }

    pub fn get_pointer(&self) -> Vec<&HardwarePointer> {
        self.pointers.values().collect::<Vec<_>>()
    }
}
