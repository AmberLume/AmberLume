use crate::legacy_render::view::render_view::RenderView;

#[derive(Clone, Copy)]
pub struct RenderViewsLayout {
    pub main: RenderView,
    pub cascade_count: u32,
}
