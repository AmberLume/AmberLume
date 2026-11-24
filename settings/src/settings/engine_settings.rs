use crate::debug_settings::DebugSettings;
use crate::editor_settings::EditorSettings;
use crate::hardware_capabilities::HardwareCapabilities;
use crate::light_settings::LightSettings;
use crate::legacy_render_settings::LegacyRenderSettings;

#[derive(Copy, Clone)]
pub struct EngineSettings {
    pub debug: DebugSettings,
    pub editor: EditorSettings,
    pub render: LegacyRenderSettings,
    pub light: LightSettings,
}

impl Default for EngineSettings {
    fn default() -> Self {
        Self {
            debug: DebugSettings::default(),
            editor: EditorSettings::default(),
            render: LegacyRenderSettings::default(),
            light: LightSettings::default(),
        }
    }
}

impl EngineSettings {
    pub fn with_hardware_defaults(mut self, capabilities: HardwareCapabilities) -> Self {
        self.render = self.render.with_hardware_defaults(capabilities);

        self
    }
}
