use crate::ui::ui_context::UiContext;

pub trait UiRenderer {
    fn render(&self, context: &UiContext);
}
