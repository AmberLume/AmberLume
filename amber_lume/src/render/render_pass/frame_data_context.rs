use crate::snapshot_handler::world_snapshot::WorldSnapshot;
use anyhow::Result;
use std::sync::Arc;
use crate::limits::renderer_limits::RendererLimits;
use crate::render::render_pass::render_pass_layout::RenderViewsLayout;
use crate::render::render_pass::ui::ui_snapshot::{UiSnapshot};

pub struct FrameDataContext<'a> {
    pub renderer_limits: &'a RendererLimits,
    
    pub render_views_layout: RenderViewsLayout,
    
    pub world_snapshot: Arc<WorldSnapshot>,
    
    pub ui_snapshot: UiSnapshot,
}

impl<'a> FrameDataContext<'a> {
    pub fn create(
        renderer_limits: &'a RendererLimits,
        render_views_layout: RenderViewsLayout,
        world_snapshot: Arc<WorldSnapshot>,
        ui_snapshot: UiSnapshot,
    ) -> Result<Self> {
        Ok(Self {
            renderer_limits,
            
            render_views_layout,
            
            world_snapshot,
            
            ui_snapshot,
        })
    }
}
