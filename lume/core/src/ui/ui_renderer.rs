use crate::ui::layouts::root_fragment_state::RootFragmentState;
use amber_lume::settings::settings_handler::EngineSettingsHandler;
use amber_lume::ui::ui_context::UiContext;
use amber_lume::ui::ui_renderer::UiRenderer;
use amber_lume::ui::ui_state::UiFragmentState;
use std::sync::Mutex;
use amber_lume::statistics::amber_lume_statistics::AmberLumeStatistics;

pub struct LumeUiRenderer {
    state: Mutex<RootFragmentState>,
}

impl LumeUiRenderer {
    pub fn new() -> Self {
        let root_fragment = RootFragmentState::create();

        Self {
            state: Mutex::new(root_fragment),
        }
    }
}

impl UiRenderer for LumeUiRenderer {
    fn render(
        &self,
        context: &UiContext,
        settings_handler: &EngineSettingsHandler,
        statistics: &AmberLumeStatistics,
    ) {
        if let Ok(mut state) = self.state.lock() {
            state.render(&context.theme, &settings_handler, &statistics);
        }
    }
}
