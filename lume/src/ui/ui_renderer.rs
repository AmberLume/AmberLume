use crate::ui::layouts::root_fragment_state::RootFragmentState;
use amber_lume::ui::ui_context::UiContext;
use amber_lume::ui::ui_renderer::UiRenderer;
use amber_lume::ui::ui_state::UiFragmentState;
use std::sync::Mutex;
use amber_lume::amber_lume::AmberLume;

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

    pub fn update(&self, amber_lume: &AmberLume) {
        if let Ok(mut state) = self.state.lock() {
            state.update(amber_lume);
        }
    }
}

impl UiRenderer for LumeUiRenderer {
    fn render(&self, context: &UiContext) {
        if let Ok(mut state) = self.state.lock() {
            state.render(context);
        }
    }
}
