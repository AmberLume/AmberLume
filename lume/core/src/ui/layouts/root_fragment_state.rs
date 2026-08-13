use crate::ui::layouts::debug_fragment_state::DebugFragmentState;
use crate::ui::layouts::editor_fragment_state::EditorFragmentState;
use crate::ui::widgets::window::window;
use input::{HardwareKeyCode, InputHandler};
use settings::EngineSettingsHandler;
use statistics::AmberLumeStatistics;
use ui::Theme;
use ui::UiFragmentState;
use yakui::column;

pub struct RootFragmentState {
    debug_fragment_state: DebugFragmentState,
    editor_fragment_state: EditorFragmentState,
}

impl RootFragmentState {
    pub fn create() -> Self {
        Self {
            debug_fragment_state: DebugFragmentState::create(),
            editor_fragment_state: EditorFragmentState::create(),
        }
    }
}

impl UiFragmentState for RootFragmentState {
    fn render(
        &mut self,
        theme: &Theme,
        input: &mut InputHandler,
        settings_handler: &EngineSettingsHandler,
        statistics: &AmberLumeStatistics,
        picked_entity: Option<u32>,
    ) {
        if input.key(HardwareKeyCode::F12, true).is_just_pressed() {
            settings_handler.update(|settings| {
                let current = settings.editor.enabled.value;

                settings.editor.enabled.set(!current);
            });
        }

        column(|| {
            window(&theme, "Debug", || {
                self.debug_fragment_state
                    .render(&theme, settings_handler, statistics);
            });
        });

        if settings_handler.settings().editor.enabled.value {
            column(|| {
                window(&theme, "Editor", || {
                    self.editor_fragment_state
                        .render(&theme, settings_handler, picked_entity);
                });
            });
        }
    }
}
