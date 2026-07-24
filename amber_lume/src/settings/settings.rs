use crate::settings::debug_settings::DebugSettings;
use crate::settings::editor_settings::EditorSettings;
use crate::settings::light_settings::LightSettings;
use crate::settings::render_settings::RenderSettings;

#[derive(Copy, Clone)]
pub struct EngineSettings {
    pub debug: DebugSettings,
    pub editor: EditorSettings,
    pub render: RenderSettings,
    pub light: LightSettings,
}

impl Default for EngineSettings {
    fn default() -> Self {
        Self {
            debug: DebugSettings::default(),
            editor: EditorSettings::default(),
            render: RenderSettings::default(),
            light: LightSettings::default(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct SwitchSetting {
    pub value: bool,

    default: bool,

    pub title: &'static str,
    pub description: &'static str,
}

impl SwitchSetting {
    pub fn new(value: bool, default: bool, title: &'static str, description: &'static str) -> Self {
        Self {
            value,

            default,

            title,
            description,
        }
    }

    pub fn set(&mut self, value: bool) {
        self.value = value;
    }

    pub fn reset(&mut self) {
        self.value = self.default;
    }
}

#[derive(Copy, Clone)]
pub struct ChoiceSetting {
    pub value: usize,

    default: usize,

    pub options: &'static [&'static str],

    pub title: &'static str,
    pub description: &'static str,
}

impl ChoiceSetting {
    pub fn new(value: usize, default: usize, options: &'static [&'static str], title: &'static str, description: &'static str) -> Self {
        Self {
            value,

            default,

            options,

            title,
            description,
        }
    }

    pub fn set(&mut self, value: usize) {
        if value < self.options.len() {
            self.value = value;
        }
    }

    pub fn reset(&mut self) {
        self.value = self.default;
    }
}

#[derive(Copy, Clone)]
pub struct RangeSetting {
    pub value: f32,

    default: f32,

    pub min: f32,
    pub max: f32,

    pub title: &'static str,
    pub description: &'static str,
}

impl RangeSetting {
    pub fn new(value: f32, default: f32, min: f32, max: f32, title: &'static str, description: &'static str) -> Self {
        Self {
            value,

            default,

            min,
            max,

            title,
            description,
        }
    }

    pub fn set(&mut self, value: f32) {
        self.value = value.clamp(self.min, self.max);
    }

    pub fn reset(&mut self) {
        self.value = self.default;
    }
}
