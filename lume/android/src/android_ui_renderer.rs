use amber_lume::settings::settings_handler::EngineSettingsHandler;
use amber_lume::statistics::amber_lume_statistics::AmberLumeStatistics;
use amber_lume::ui::ui_context::UiContext;
use amber_lume::ui::ui_renderer::UiRenderer;
use amber_lume::ui::ui_state::UiFragmentState;
use core::ui::layouts::root_fragment_state::RootFragmentState;
use core::ui::widgets::clickable::clickable;
use core::ui::widgets::window::window;
use std::sync::Arc;
use parking_lot::Mutex;
use yakui::{column, pad, row, text};
use yakui::widgets::Pad;
use input::HardwareKeyCode;
use amber_lume::editor::editor_state::EditorState;
use crate::input_handler::InputHandler;

pub struct AndroidUiRenderer {
    state: Mutex<RootFragmentState>,
    input_handler: Arc<InputHandler>,
}

impl AndroidUiRenderer {
    pub fn new(
        input_handler: Arc<InputHandler>,
    ) -> Self {
        let root_fragment = RootFragmentState::create();

        Self {
            state: Mutex::new(root_fragment),
            input_handler,
        }
    }

    fn pad_button(&self, glyph: &'static str, keycode: HardwareKeyCode) {
        let response = clickable(|| {
            pad(Pad::all(32.0), || {
                text(64.0, glyph);
            });
        });

        self.input_handler.push(keycode, response.pressed)
    }
}

impl UiRenderer for AndroidUiRenderer {
    fn render(
        &self,
        context: &UiContext,
        input: &mut input::InputHandler,
        settings_handler: &EngineSettingsHandler,
        statistics: &AmberLumeStatistics,
        editor_state: &EditorState,
    ) {
        let mut state = self.state.lock();
        state.render(&context.theme, input, &settings_handler, &statistics, &editor_state);

        column(|| {
            window(&context.theme, "Control", || {
                column(|| {
                    self.pad_button("/\\", HardwareKeyCode::W);

                    row(|| {
                        self.pad_button("<", HardwareKeyCode::A);
                        self.pad_button(">", HardwareKeyCode::D);
                    });

                    self.pad_button("\\/", HardwareKeyCode::S);

                    self.pad_button("C", HardwareKeyCode::Space);
                });
            });
        });
    }
}
