use amber_lume::input_handler::input_frame::InputFrame;
use amber_lume::settings::settings_handler::EngineSettingsHandler;
use amber_lume::statistics::amber_lume_statistics::AmberLumeStatistics;
use amber_lume::ui::theme::Theme;
use amber_lume::ui::ui_state::UiFragmentState;

pub struct EditorFragmentState {

}

impl EditorFragmentState {
    pub fn create() -> Self {
        Self {

        }
    }
}

impl UiFragmentState for EditorFragmentState {
    fn render(
        &mut self,
        _theme: &Theme,
        _input_frame: &InputFrame,
        _settings_handler: &EngineSettingsHandler,
        _statistics: &AmberLumeStatistics,
    ) {

    }
}
