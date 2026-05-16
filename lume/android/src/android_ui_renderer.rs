use amber_lume::settings::settings_handler::EngineSettingsHandler;
use amber_lume::statistics::amber_lume_statistics::AmberLumeStatistics;
use amber_lume::ui::ui_context::UiContext;
use amber_lume::ui::ui_renderer::UiRenderer;
use amber_lume::ui::ui_state::UiFragmentState;
use core::ui::layouts::root_fragment_state::RootFragmentState;
use core::ui::widgets::clickable::clickable;
use core::ui::widgets::window::window;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;
use yakui::{column, pad, row, text, Rect, Vec2};
use yakui::widgets::Pad;
use amber_lume::input_handler::hardware_key_codes::HardwareKeyCode;
use amber_lume::input_handler::hardware_pointer_key_codes::HardwarePointerKeyCodes;
use amber_lume::input_handler::input_frame::{InputFrame, PointerId};
use crate::input_handler::InputHandler;

#[derive(Default)]
struct TouchButtons {
    holding: HashMap<HardwareKeyCode, PointerId>,
}

impl TouchButtons {
    fn pressed(&mut self, keycode: HardwareKeyCode, rect: Option<Rect>, input_frame: &InputFrame) -> bool {
        if let Some(pid) = self.holding.get(&keycode).copied() {
            if let Some(pointer) = input_frame.get_pointer_by_id(&pid) {
                if pointer.is_down(HardwarePointerKeyCodes::Left) {
                    let outside = match (rect, pointer.position) {
                        (Some(rect), Some((x, y))) => !rect.contains_point(Vec2::new(x, y)),
                        _ => false,
                    };

                    if !outside {
                        return true;
                    }
                }
            }

            self.holding.remove(&keycode);
        }

        let Some(rect) = rect else {
            return false;
        };

        for (pid, pointer) in input_frame.pointers_with_ids() {
            if !pointer.just_pressed(HardwarePointerKeyCodes::Left) {
                continue;
            }

            let Some((x, y)) = pointer.position else {
                continue;
            };

            if rect.contains_point(Vec2::new(x, y)) && !self.holding.values().any(|h| h == pid) {
                self.holding.insert(keycode, *pid);

                return true;
            }
        }

        false
    }
}

pub struct AndroidUiRenderer {
    state: Mutex<RootFragmentState>,
    touch_buttons: Mutex<TouchButtons>,
    input_handler: Arc<InputHandler>,
}

impl AndroidUiRenderer {
    pub fn new(
        input_handler: Arc<InputHandler>,
    ) -> Self {
        let root_fragment = RootFragmentState::create();

        Self {
            state: Mutex::new(root_fragment),
            touch_buttons: Mutex::new(TouchButtons::default()),
            input_handler,
        }
    }

    fn pad_button(
        &self,
        context: &UiContext,
        input_frame: &InputFrame,
        glyph: &'static str,
        keycode: HardwareKeyCode,
    ) {
        let response = clickable(|| {
            pad(Pad::all(32.0), || {
                text(64.0, glyph);
            });
        });

        let rect = context.widget_rect(response.id);
        let pressed = self.touch_buttons.lock().pressed(keycode, rect, input_frame);

        self.input_handler.push(keycode, pressed)
    }
}

impl UiRenderer for AndroidUiRenderer {
    fn render(
        &self,
        context: &UiContext,
        input_frame: &InputFrame,
        settings_handler: &EngineSettingsHandler,
        statistics: &AmberLumeStatistics,
    ) {
        let mut state = self.state.lock();
        state.render(&context.theme, input_frame, &settings_handler, &statistics);

        column(|| {
            window(&context.theme, "Control", || {
                column(|| {
                    self.pad_button(context, input_frame, "/\\", HardwareKeyCode::W);

                    row(|| {
                        self.pad_button(context, input_frame, "<", HardwareKeyCode::A);
                        self.pad_button(context, input_frame, ">", HardwareKeyCode::D);
                    });

                    self.pad_button(context, input_frame, "\\/", HardwareKeyCode::S);

                    self.pad_button(context, input_frame, "C", HardwareKeyCode::Space);
                });
            });
        });
    }
}
