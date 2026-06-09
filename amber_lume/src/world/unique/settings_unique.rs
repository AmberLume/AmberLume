use std::sync::Arc;
use arc_swap::ArcSwap;
use shipyard::Unique;
use crate::settings::settings::EngineSettings;

#[derive(Unique)]
pub struct SettingsUnique {
    pub settings: Arc<ArcSwap<EngineSettings>>,
}

impl SettingsUnique {
    pub fn new(settings: Arc<ArcSwap<EngineSettings>>) -> Self {
        Self { settings }
    }
}
