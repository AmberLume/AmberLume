use crate::render::pass::pass_context::PassContext;
use anyhow::Result;
use crate::ids::FrameIndex;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;

pub trait Pass {
    type PassData;
    type Statistics;
    
    fn is_enabled(&self) -> bool;

    fn prepare_data(&self, context: &FrameDataContext) -> Result<Self::PassData>;

    fn record_commands(&self, render_pass_context: &PassContext, data: Self::PassData) -> Result<()>;

    fn statistics(&self, frame_index: FrameIndex) -> Self::Statistics;
    
    fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> where Self: Sized;
}
