use crate::settings::debug_settings::DebugSettings;
use crate::settings::input_settings::InputSettings;

#[derive(Copy, Clone)]
pub struct EngineSettings {
    pub debug: DebugSettings,
    pub input: InputSettings,
}

impl Default for EngineSettings {
    fn default() -> Self {
        Self {
            debug: DebugSettings::default(),
            input: InputSettings::default(),
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
