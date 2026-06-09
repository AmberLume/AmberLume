use yakui::Color;
use yakui::widgets::Text;
use amber_lume::editor::editor_state::EditorState;
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
        editor_state: &EditorState,
    ) {
        let value = match editor_state.picked_entity {
            Some(entity) => format!("Picked: {}", entity),
            None => String::from("Picked: -"),
        };

        let mut text = Text::new(16.0, value);
        text.style.color = Color::WHITE;
        text.show();
    }
}
