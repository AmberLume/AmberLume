use yakui::Color;
use yakui::widgets::Text;
use amber_lume::editor::editor_state::EditorState;
use settings::EngineSettingsHandler;
use amber_lume::statistics::amber_lume_statistics::AmberLumeStatistics;
use amber_lume::ui::theme::Theme;

pub struct EditorFragmentState {

}

impl EditorFragmentState {
    pub fn create() -> Self {
        Self {

        }
    }

    pub fn render(
        &mut self,
        _theme: &Theme,
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
