use std::sync::Arc;
use glam::Mat4;
use index_allocator::FrameIndex;
use crate::limits::RenderLimits;
use settings::RenderSettings;
use crate::render::pass::pass_layout::RenderViewsLayout;
use crate::render::pass::ui::ui_frame::UiFrame;
use render_snapshot::RenderSnapshot;

pub struct FrameDataContext<'pass_prepare> {
    pub frame_index: FrameIndex,

    pub limits: &'pass_prepare RenderLimits,

    pub render_settings: RenderSettings,

    pub render_views_layout: &'pass_prepare RenderViewsLayout,

    pub render_snapshot: Arc<RenderSnapshot>,

    pub previous_transforms: Vec<Mat4>,

    pub ui_frame: UiFrame,
}

impl<'pass_prepare> FrameDataContext<'pass_prepare> {
    pub fn create(
        frame_index: FrameIndex,
        limits: &'pass_prepare RenderLimits,
        render_settings: RenderSettings,
        render_views_layout: &'pass_prepare RenderViewsLayout,
        render_snapshot: Arc<RenderSnapshot>,
        previous_transforms: Vec<Mat4>,
        ui_frame: UiFrame,
    ) -> Self {
        Self {
            frame_index,

            limits,

            render_settings,

            render_views_layout,

            render_snapshot,

            previous_transforms,

            ui_frame,
        }
    }
}
