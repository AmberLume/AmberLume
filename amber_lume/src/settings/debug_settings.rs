use crate::settings::settings::{ChoiceSetting, SwitchSetting};

pub const DEBUG_LAYER_OPTIONS: &[&str] = &["Off", "Velocity", "Normal", "GTAO"];

#[derive(Copy, Clone)]
pub struct DebugSettings {
    pub collider_rendering_enabled: SwitchSetting,
    pub physics_interpolation: SwitchSetting,
    pub physics_paused: SwitchSetting,
    pub debug_layer: ChoiceSetting,
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
            debug_layer: ChoiceSetting::new(
                0,
                0,
                DEBUG_LAYER_OPTIONS,
                "Debug layer",
                "Render a selected intermediate render layer fullscreen instead of the final image.",
            ),
        }
    }
}
