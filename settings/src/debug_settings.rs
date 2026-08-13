use crate::settings::switch_setting::SwitchSetting;

#[derive(Copy, Clone)]
pub struct DebugSettings {
    pub physics_interpolation: SwitchSetting,
    pub physics_paused: SwitchSetting,
}

impl Default for DebugSettings {
    fn default() -> Self {
        Self {
            physics_interpolation: SwitchSetting::new(true, "Physics interpolation", "..."),
            physics_paused: SwitchSetting::new(false, "Physics paused", "..."),
        }
    }
}
