use crate::ui::layouts::debug_fragment_state::DebugFragmentState;
use crate::ui::layouts::editor_fragment_state::EditorFragmentState;
use crate::ui::widgets::window::window;
use amber_lume::settings::settings_handler::EngineSettingsHandler;
use amber_lume::statistics::amber_lume_statistics::AmberLumeStatistics;
use amber_lume::ui::theme::Theme;
use amber_lume::ui::ui_state::UiFragmentState;
use yakui::column;
use amber_lume::editor::editor_state::EditorState;
use amber_lume::input_handler::hardware_key_codes::HardwareKeyCode;
use amber_lume::input_handler::input_frame::InputFrame;

pub struct RootFragmentState {
    pub debug_fragment_state: DebugFragmentState,
    pub editor_fragment_state: EditorFragmentState,
}

impl RootFragmentState {
    pub fn create() -> Self {
        let debug_fragment_state = DebugFragmentState::create();
        let editor_fragment_state = EditorFragmentState::create();

        Self {
            debug_fragment_state,
            editor_fragment_state,
        }
    }
}

impl UiFragmentState for RootFragmentState {
    fn render(
        &mut self,
        theme: &Theme,
        input_frame: &InputFrame,
        settings_handler: &EngineSettingsHandler,
        statistics: &AmberLumeStatistics,
        editor_state: &EditorState,
    ) {
        if input_frame.just_pressed(HardwareKeyCode::F3) {
            settings_handler.update(|settings| {
                let current = settings.input.cursor_controls_camera.value;

                settings.input.cursor_controls_camera.set(!current);
            });

            settings_handler.apply();
        }

        if input_frame.just_pressed(HardwareKeyCode::F12) {
            settings_handler.update(|settings| {
                let current = settings.editor.enabled.value;

                settings.editor.enabled.set(!current);
            });

            settings_handler.apply();
        }

        column(|| {
            window(&theme, "Debug", || {
                self.debug_fragment_state
                    .render(&theme, &input_frame, &settings_handler, &statistics, &editor_state);
            });
        });

        if settings_handler.get_pending().editor.enabled.value {
            column(|| {
                window(&theme, "Editor", || {
                    self.editor_fragment_state
                        .render(&theme, &input_frame, &settings_handler, &statistics, &editor_state);
                });
            });
        }
    }
}
