use crate::store::blas_queue::blas_event::BlasEvent;
use parking_lot::Mutex;
use std::mem::take;

pub struct BlasQueue {
    events: Mutex<Vec<BlasEvent>>,
}

impl BlasQueue {
    pub fn new() -> Self {
        Self {
            events: Mutex::new(Vec::new()),
        }
    }

    pub fn push(&self, event: BlasEvent) {
        self.events.lock().push(event);
    }

    pub fn drain(&self) -> Vec<BlasEvent> {
        take(&mut *self.events.lock())
    }
}
