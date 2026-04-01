use std::cell::Cell;
use std::time::Instant;
use tracing::warn;

pub struct TimeMeasurement {
    instant: Cell<Option<Instant>>,

    pub result: Cell<u64>,
}

impl TimeMeasurement {
    pub fn new() -> Self {
        Self {
            instant: Cell::new(None),

            result: Cell::new(0),
        }
    }

    pub fn start(&self) {
        self.instant.set(Some(Instant::now()));
    }

    pub fn finish(&self) {
        if let Some(instant) = self.instant.get() {
            self.result.set(instant.elapsed().as_nanos() as u64);
            self.instant.set(None);
        } else {
            warn!("Called finish() before start()");
        }
    }

    pub fn collect(&self) -> u64 {
        self.result.get()
    }
}
