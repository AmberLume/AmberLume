use crate::settings::settings_handler::EngineSettingsHandler;
use crate::ui::ui_context::UiContext;

pub trait UiRenderer {
    fn render(&self, context: &UiContext, settings_handler: &EngineSettingsHandler);
}
