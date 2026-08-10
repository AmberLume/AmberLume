use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use arc_swap::ArcSwap;
use parking_lot::Mutex;
use crate::settings::EngineSettings;

pub struct EngineSettingsHandler {
    current: Arc<ArcSwap<EngineSettings>>,

    pending_updated: AtomicBool,
    pending: EngineSettings,
    pending_mut: Arc<Mutex<EngineSettings>>,

    apply_called: AtomicBool,
}

impl EngineSettingsHandler {
    pub fn new(settings: EngineSettings) -> Self {
        Self {
            current: Arc::new(ArcSwap::from(Arc::new(settings))),

            pending_updated: AtomicBool::new(false),
            pending: settings,
            pending_mut: Arc::new(Mutex::new(settings)),

            apply_called: AtomicBool::new(false),
        }
    }

    pub fn get_current(&self) -> Arc<ArcSwap<EngineSettings>> {
        self.current.clone()
    }

    pub fn get_pending(&self) -> EngineSettings {
        self.pending
    }

    pub fn update(&self, modify: impl FnOnce(&mut EngineSettings)) {
        modify(&mut self.pending_mut.lock());

        self.pending_updated.store(true, Ordering::Relaxed);
    }

    pub fn reset(&self) {
        let mut pending_mut = self.pending_mut.lock();

        *pending_mut = **self.current.load();

        self.pending_updated.store(true, Ordering::Relaxed);
    }

    pub fn flush(&mut self) {
        if self.pending_updated.swap(false, Ordering::Relaxed) {
            let pending = self.pending_mut.lock();

            self.pending = *pending;
        }

        if self.apply_called.swap(false, Ordering::Relaxed) {
            self.current.store(Arc::new(self.pending));
        }
    }

    pub fn apply(&self) {
        self.apply_called.store(true, Ordering::Relaxed)
    }
}
