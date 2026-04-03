use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_graph::image_state_tracker::image_state_tracker::ImageStateTracker;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use anyhow::Result;

pub trait PassEntry {
    fn run(
        &mut self,
        frame_data_context: &FrameDataContext,
        passed_context: &PassContext,
        declaration: &mut PassResourceDeclaration,
        image_state_tracker: &mut ImageStateTracker,
    ) -> Result<()>;

    fn destroy(self: Box<Self>, resource_factories: &ResourceFactories) -> Result<()>;
}
