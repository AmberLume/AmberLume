use input::InputHandler;
use settings::EngineSettingsHandler;
use statistics::AmberLumeStatistics;
use ui::UiContext;
use ui::UiRenderer;
use ui::UiFragmentState;
use core::ui::layouts::root_fragment_state::RootFragmentState;
use parking_lot::Mutex;

pub struct DesktopUiRenderer {
    state: Mutex<RootFragmentState>,
}

impl DesktopUiRenderer {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(RootFragmentState::create()),
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
        picked_entity: Option<u32>,
    ) {
        self.state.lock().render(
            &context.theme,
            input,
            settings_handler,
            statistics,
            picked_entity,
        );
    }
}
