use crate::amber_lume::AmberLume;
use crate::settings::settings_handler::EngineSettingsHandler;
use crate::ui::theme::Theme;

pub trait UiFragmentState {
    fn update(&mut self, amber_lume: &AmberLume);

    fn render(&mut self, theme: &Theme, settings_handler: &EngineSettingsHandler);
}
