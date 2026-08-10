use crate::ui_draw_layer::UiDrawLayer;
use crate::ui_vertex::UiVertex;

#[derive(Clone)]
pub struct UiFrame {
    pub indices: Vec<u32>,
    pub vertices: Vec<UiVertex>,

    pub draw_layers: Vec<UiDrawLayer>,
}
