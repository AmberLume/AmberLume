use crate::render::render_pass::render_pass_context::RenderPassContext;
use anyhow::Result;
use crate::render::render_pass::frame_data_context::FrameDataContext;

pub trait RenderPass {
    type RenderPassData;

    fn is_enabled(&self) -> bool;

    fn prepare_data(&self, context: &FrameDataContext) -> Result<Self::RenderPassData>;

    fn record_commands(&self, render_pass_context: &RenderPassContext, data: Self::RenderPassData) -> Result<()>;

    fn destroy(self) -> Result<()> where Self: Sized {
        Ok(())
    }
}
