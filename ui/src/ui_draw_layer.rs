use crate::ui_draw_call::UiDrawCall;

#[derive(Clone)]
pub struct UiDrawLayer {
    pub draw_calls: Vec<UiDrawCall>,
}
