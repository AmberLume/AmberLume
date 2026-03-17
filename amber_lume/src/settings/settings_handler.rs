use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use arc_swap::ArcSwap;
use parking_lot::Mutex;
use crate::settings::settings::EngineSettings;

pub struct EngineSettingsHandler {
    current: Arc<ArcSwap<EngineSettings>>,

    pending_updated: AtomicBool,
    pending: EngineSettings,
    pending_mut: Arc<Mutex<EngineSettings>>,
}

impl EngineSettingsHandler {
    pub fn new(settings: EngineSettings) -> Self {
        Self {
            current: Arc::new(ArcSwap::from(Arc::new(settings))),

            pending_updated: AtomicBool::new(false),
            pending: settings,
            pending_mut: Arc::new(Mutex::new(settings)),
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

    pub fn flush(&mut self) {
        if self.pending_updated.swap(false, Ordering::Relaxed) {
            let pending = self.pending_mut.lock();

            self.pending = *pending;
        }
    }

    pub fn apply(&mut self) {
        self.current.store(Arc::new(self.pending));
    }
}
