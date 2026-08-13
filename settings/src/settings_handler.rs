use std::sync::Arc;
use arc_swap::ArcSwap;
use crate::settings::engine_settings::EngineSettings;

pub struct EngineSettingsHandler {
    current: Arc<ArcSwap<EngineSettings>>,
}

impl EngineSettingsHandler {
    pub fn new(settings: EngineSettings) -> Self {
        Self {
            current: Arc::new(ArcSwap::from(Arc::new(settings))),
        }
    }

    pub fn current(&self) -> Arc<ArcSwap<EngineSettings>> {
        self.current.clone()
    }

    pub fn settings(&self) -> EngineSettings {
        **self.current.load()
    }

    pub fn update(&self, modify: impl FnOnce(&mut EngineSettings)) {
        let mut settings = **self.current.load();

        modify(&mut settings);

        self.current.store(Arc::new(settings));
    }
}
