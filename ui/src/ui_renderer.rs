use input::InputHandler;
use settings::EngineSettingsHandler;
use statistics::AmberLumeStatistics;
use crate::ui_context::UiContext;

pub trait UiRenderer {
    fn render(
        &self,
        context: &UiContext,
        input: &mut InputHandler,
        settings_handler: &EngineSettingsHandler,
        statistics: &AmberLumeStatistics,
        picked_entity: Option<u32>,
    );
}
