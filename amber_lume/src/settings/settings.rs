use crate::settings::debug_settings::DebugSettings;
use crate::settings::editor_settings::EditorSettings;
use crate::settings::input_settings::InputSettings;
use crate::settings::render_settings::RenderSettings;

#[derive(Copy, Clone)]
pub struct EngineSettings {
    pub debug: DebugSettings,
    pub input: InputSettings,
    pub editor: EditorSettings,
    pub render: RenderSettings,
}

impl Default for EngineSettings {
    fn default() -> Self {
        Self {
            debug: DebugSettings::default(),
            input: InputSettings::default(),
            editor: EditorSettings::default(),
            render: RenderSettings::default(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct SwitchSetting {
    value: bool,

    default: bool,

    title: &'static str,
    description: &'static str,
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

    pub fn get(&self) -> bool {
        self.value
    }

    pub fn get_title(&self) -> &'static str {
        self.title
    }

    pub fn get_description(&self) -> &'static str {
        self.description
    }

    pub fn set(&mut self, value: bool) {
        self.value = value;
    }

    pub fn reset(&mut self) {
        self.value = self.default;
    }
}

#[derive(Copy, Clone)]
pub struct RangeSetting {
    value: f32,

    default: f32,

    min: f32,
    max: f32,

    title: &'static str,
    description: &'static str,
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

    pub fn get(&self) -> f32 {
        self.value
    }

    pub fn get_min(&self) -> f32 {
        self.min
    }

    pub fn get_max(&self) -> f32 {
        self.max
    }

    pub fn get_title(&self) -> &'static str {
        self.title
    }

    pub fn get_description(&self) -> &'static str {
        self.description
    }

    pub fn set(&mut self, value: f32) {
        self.value = value.clamp(self.min, self.max);
    }

    pub fn reset(&mut self) {
        self.value = self.default;
    }
}
