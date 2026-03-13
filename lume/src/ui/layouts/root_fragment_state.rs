use yakui::Vec2;
use amber_lume::amber_lume::AmberLume;
use amber_lume::ui::ui_context::UiContext;
use amber_lume::ui::ui_state::UiFragmentState;
use crate::ui::layouts::debug_fragment_state::DebugFragmentState;
use crate::ui::widgets::window::{window, WindowContext};

pub struct RootFragmentState {
    pub debug_window_context: WindowContext,

    pub debug_fragment_state: DebugFragmentState,
}

impl RootFragmentState {
    pub fn create() -> Self {
        let debug_fragment_state = DebugFragmentState::create();
        
        Self {
            debug_window_context: WindowContext {
                title: String::from("Debug"),
                
                position: Vec2::new(0.0, 0.0),
                size: Vec2::new(200.0, 200.0),
            },

            debug_fragment_state,
        }
    }
}

impl UiFragmentState for RootFragmentState {
    fn update(&mut self, amber_lume: &AmberLume) {
        self.debug_fragment_state.update(amber_lume)
    }
    
    fn render(&mut self, context: &UiContext) {
        window(&context, &mut self.debug_window_context, || {
            self.debug_fragment_state.render(&context);
        });
    }
}
