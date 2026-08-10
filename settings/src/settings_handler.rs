use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use arc_swap::ArcSwap;
use parking_lot::Mutex;
use crate::settings::EngineSettings;

pub struct EngineSettingsHandler {
    current: Arc<ArcSwap<EngineSettings>>,

    pending: Mutex<EngineSettings>,

    apply_called: AtomicBool,
}

impl EngineSettingsHandler {
    pub fn new(settings: EngineSettings) -> Self {
        Self {
            current: Arc::new(ArcSwap::from(Arc::new(settings))),

            pending: Mutex::new(settings),

            apply_called: AtomicBool::new(false),
        }
    }

    pub fn get_current(&self) -> Arc<ArcSwap<EngineSettings>> {
        self.current.clone()
    }

    pub fn get_pending(&self) -> EngineSettings {
        *self.pending.lock()
    }

    pub fn update(&self, modify: impl FnOnce(&mut EngineSettings)) {
        modify(&mut self.pending.lock());
    }

    pub fn reset(&self) {
        *self.pending.lock() = **self.current.load();
    }

    pub fn flush(&self) {
        if self.apply_called.swap(false, Ordering::Relaxed) {
            self.current.store(Arc::new(*self.pending.lock()));
        }
    }

    pub fn apply(&self) {
        self.apply_called.store(true, Ordering::Relaxed)
    }
}
