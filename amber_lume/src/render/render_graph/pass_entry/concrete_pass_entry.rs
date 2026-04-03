use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_graph::image_state_tracker::image_state_tracker::ImageStateTracker;
use crate::render::render_graph::pass_entry::pass_entry::PassEntry;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use anyhow::Result;
use crate::render::factories::resource_factories::ResourceFactories;

pub struct ConcretePassEntry<P: Pass> {
    pub pass: P,
}

impl<P: Pass> ConcretePassEntry<P> {
    pub fn new(pass: P) -> Self {
        Self { pass }
    }
}

impl<P: Pass> PassEntry for ConcretePassEntry<P> {
    fn run(
        &mut self,
        frame_data_context: &FrameDataContext,
        pass_context: &PassContext,
        declaration: &mut PassResourceDeclaration,
        image_state_tracker: &mut ImageStateTracker,
    ) -> Result<()> {
        if !self.pass.is_enabled() {
            return Ok(());
        }

        declaration.clear();
        self.pass.declare_resources(&pass_context, declaration);
        declaration.apply(image_state_tracker);
        image_state_tracker.flush(&pass_context);

        let data = self.pass.prepare_data(frame_data_context)?;
        self.pass.record_commands(&pass_context, data)?;

        image_state_tracker.flush(&pass_context);

        Ok(())
    }

    fn destroy(self: Box<Self>, resource_factories: &ResourceFactories) -> Result<()> {
        self.pass.destroy(&resource_factories)
    }
}
