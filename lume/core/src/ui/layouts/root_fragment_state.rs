use yakui::column;
use amber_lume::settings::settings_handler::EngineSettingsHandler;
use amber_lume::statistics::amber_lume_statistics::AmberLumeStatistics;
use amber_lume::ui::theme::Theme;
use amber_lume::ui::ui_state::UiFragmentState;
use crate::ui::layouts::debug_fragment_state::DebugFragmentState;
use crate::ui::widgets::window::window;

pub struct RootFragmentState {
    pub debug_fragment_state: DebugFragmentState,
}

impl RootFragmentState {
    pub fn create() -> Self {
        let debug_fragment_state = DebugFragmentState::create();
        
        Self {
            debug_fragment_state,
        }
    }
}

impl UiFragmentState for RootFragmentState {
    fn render(&mut self, theme: &Theme, settings_handler: &EngineSettingsHandler, statistics: &AmberLumeStatistics,) {
        column(|| {
            window(&theme, "Debug", || {
                self.debug_fragment_state.render(&theme, &settings_handler, &statistics);
            });
        });
    }
}
