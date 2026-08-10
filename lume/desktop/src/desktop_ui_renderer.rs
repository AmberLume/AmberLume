use amber_lume::editor::editor_state::EditorState;
use input::InputHandler;
use settings::EngineSettingsHandler;
use amber_lume::statistics::amber_lume_statistics::AmberLumeStatistics;
use amber_lume::ui::ui_context::UiContext;
use amber_lume::ui::ui_renderer::UiRenderer;
use amber_lume::ui::ui_state::UiFragmentState;
use core::ui::layouts::root_fragment_state::RootFragmentState;
use parking_lot::Mutex;

pub struct DesktopUiRenderer {
    state: Mutex<RootFragmentState>,
}

impl DesktopUiRenderer {
    pub fn new() -> Self {
        let root_fragment = RootFragmentState::create();

        Self {
            state: Mutex::new(root_fragment),
        }
    }
}

impl UiRenderer for DesktopUiRenderer {
    fn render(
        &self,
        context: &UiContext,
        input: &mut InputHandler,
        settings_handler: &EngineSettingsHandler,
        statistics: &AmberLumeStatistics,
        editor_state: &EditorState,
    ) {
        self.state.lock().render(
            &context.theme,
            input,
            &settings_handler,
            &statistics,
            &editor_state,
        );
    }
}
