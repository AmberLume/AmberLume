use crate::settings::settings::SwitchSetting;

#[derive(Copy, Clone)]
pub struct EditorSettings {
    pub enabled: SwitchSetting,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            enabled: SwitchSetting::new(false, false, "Editor enabled", "..."),
        }
    }
}
