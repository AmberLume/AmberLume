use crate::input_handler::input_frame::InputFrame;
use crate::settings::settings_handler::EngineSettingsHandler;
use crate::statistics::amber_lume_statistics::AmberLumeStatistics;
use crate::ui::theme::Theme;

pub trait UiFragmentState {
    fn render(
        &mut self,
        theme: &Theme,
        input_frame: &InputFrame,
        settings_handler: &EngineSettingsHandler,
        statistics: &AmberLumeStatistics,
    );
}
