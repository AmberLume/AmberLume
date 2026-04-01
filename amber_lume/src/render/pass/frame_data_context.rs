use std::sync::Arc;
use crate::limits::renderer_limits::RendererLimits;
use crate::render::pass::pass_layout::RenderViewsLayout;
use crate::render::pass::ui::ui_snapshot::{UiSnapshot};
use crate::snapshot_handler::render_snapshot::RenderSnapshot;

pub struct FrameDataContext<'a> {
    pub renderer_limits: &'a RendererLimits,

    pub render_views_layout: &'a RenderViewsLayout,

    pub render_snapshot: Arc<RenderSnapshot>,
    
    pub ui_snapshot: UiSnapshot,
}

impl<'a> FrameDataContext<'a> {
    pub fn create(
        renderer_limits: &'a RendererLimits,
        render_views_layout: &'a RenderViewsLayout,
        render_snapshot: Arc<RenderSnapshot>,
        ui_snapshot: UiSnapshot,
    ) -> Self {
        Self {
            renderer_limits,

            render_views_layout,

            render_snapshot,
            
            ui_snapshot,
        }
    }
}
