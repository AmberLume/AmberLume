use input::InputHandler;
use settings::EngineSettingsHandler;
use statistics::AmberLumeStatistics;
use crate::theme::Theme;

pub trait UiFragmentState {
    fn render(
        &mut self,
        theme: &Theme,
        input: &mut InputHandler,
        settings_handler: &EngineSettingsHandler,
        statistics: &AmberLumeStatistics,
        picked_entity: Option<u32>,
    );
}
