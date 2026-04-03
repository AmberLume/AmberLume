use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_graph::image_state_tracker::image_state_tracker::ImageStateTracker;
use crate::render::render_graph::pass_entry::concrete_pass_entry::ConcretePassEntry;
use crate::render::render_graph::pass_entry::pass_entry::PassEntry;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use anyhow::Result;
use crate::render::statistics::pass_profiler::PassProfiler;

pub struct PassGraph {
    passes: Vec<Box<dyn PassEntry>>,
    declaration: PassResourceDeclaration,
}

impl PassGraph {
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
            declaration: PassResourceDeclaration::new(),
        }
    }

    pub fn add_pass<P: Pass + 'static>(&mut self, pass: P) {
        self.passes.push(Box::new(ConcretePassEntry::new(pass)));
    }

    pub fn run(
        &mut self,
        frame_data_context: &FrameDataContext,
        pass_context: &PassContext,
        image_state_tracker: &mut ImageStateTracker,
        pass_profiler: &mut PassProfiler,
    ) -> Result<()> {
        for entry in &mut self.passes {
            entry.run(
                frame_data_context,
                pass_context,
                &mut self.declaration,
                image_state_tracker,
                pass_profiler,
            )?;
        }

        Ok(())
    }

    pub fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> {
        for entry in self.passes {
            entry.destroy(resource_factories)?;
        }

        Ok(())
    }
}
