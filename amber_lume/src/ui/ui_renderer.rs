use input::InputHandler;
use crate::editor::editor_state::EditorState;
use settings::EngineSettingsHandler;
use crate::statistics::amber_lume_statistics::AmberLumeStatistics;
use crate::ui::ui_context::UiContext;

pub trait UiRenderer {
    fn render(
        &self,
        context: &UiContext,
        input: &mut InputHandler,
        settings_handler: &EngineSettingsHandler,
        statistics: &AmberLumeStatistics,
        editor_state: &EditorState,
    );
}
