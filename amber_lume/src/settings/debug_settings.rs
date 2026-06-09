use crate::settings::settings::SwitchSetting;

#[derive(Copy, Clone)]
pub struct DebugSettings {
    pub collider_rendering_enabled: SwitchSetting,
    pub physics_interpolation: SwitchSetting,
    pub physics_paused: SwitchSetting,
}

impl Default for DebugSettings {
    fn default() -> Self {
        Self {
            collider_rendering_enabled: SwitchSetting::new(
                false,
                false,
                "Collider rendering enabled",
                "...",
            ),
            physics_interpolation: SwitchSetting::new(true, true, "Physics interpolation", "..."),
            physics_paused: SwitchSetting::new(false, false, "Physics paused", "..."),
        }
    }
}
