use crate::render::vulkan::render_pass::render_pass_context::RenderPassContext;
use anyhow::Result;

pub trait RenderPass {
    fn is_enabled(&self) -> bool;

    fn begin_record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()>;

    fn record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()>;

    fn end_record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()>;

    fn destroy(&self) -> Result<()> {
        Ok(())
    }
}
