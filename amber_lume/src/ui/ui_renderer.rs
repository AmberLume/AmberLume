use crate::settings::settings_handler::EngineSettingsHandler;
use crate::statistics::amber_lume_statistics::AmberLumeStatistics;
use crate::ui::ui_context::UiContext;

pub trait UiRenderer {
    fn render(
        &self,
        context: &UiContext,
        settings_handler: &EngineSettingsHandler,
        statistics: &AmberLumeStatistics,
    );
}
