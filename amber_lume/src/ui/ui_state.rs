use crate::amber_lume::AmberLume;
use crate::ui::ui_context::UiContext;

pub trait UiFragmentState {
    fn update(&mut self, amber_lume: &AmberLume);

    fn render(&mut self, context: &UiContext);
}
