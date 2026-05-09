use crate::input_handler::input_frame::InputFrame;
use crate::settings::settings_handler::EngineSettingsHandler;
use crate::statistics::amber_lume_statistics::AmberLumeStatistics;
use crate::ui::ui_context::UiContext;

pub trait UiRenderer {
    fn render(
        &self,
        context: &UiContext,
        input_frame: &InputFrame,
        settings_handler: &EngineSettingsHandler,
        statistics: &AmberLumeStatistics,
    );
}
