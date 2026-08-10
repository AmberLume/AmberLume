use input::InputHandler;
use crate::editor::editor_state::EditorState;
use settings::EngineSettingsHandler;
use crate::statistics::amber_lume_statistics::AmberLumeStatistics;
use crate::ui::theme::Theme;

pub trait UiFragmentState {
    fn render(
        &mut self,
        theme: &Theme,
        input: &mut InputHandler,
        settings_handler: &EngineSettingsHandler,
        statistics: &AmberLumeStatistics,
        editor_state: &EditorState,
    );
}
