use std::mem::take;
use std::sync::Mutex;
use amber_lume::input_handler::input_event::KeyEvent;

#[derive(Default)]
pub struct InputHandler {
    actions: Mutex<Vec<KeyEvent>>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&self, action: KeyEvent) {
        self.actions
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(action);
    }

    pub fn drain(&self) -> Vec<KeyEvent> {
        take(&mut *self.actions.lock().unwrap_or_else(|e| e.into_inner()))
    }
}