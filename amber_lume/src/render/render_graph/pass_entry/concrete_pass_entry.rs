use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_graph::resource_state_tracker::resource_state_tracker::ResourceStateTracker;
use crate::render::render_graph::pass_entry::pass_entry::PassEntry;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use anyhow::Result;
use crate::render::render_graph::resource_registry::resource_registry::ResourceRegistry;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::statistics::pass_profiler::PassProfiler;

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
        resource_state_tracker: &mut ResourceStateTracker,
        resource_registry: &mut ResourceRegistry,
        pass_profiler: &mut PassProfiler,
        allocator: &mut HeapAllocator,
    ) -> Result<()> {
        if !self.pass.is_enabled() {
            return Ok(());
        }

        declaration.clear();
        self.pass.declare_resources(declaration);

        pass_profiler.prepare_start(&self.pass);
        let data = self.pass.prepare_data(frame_data_context, resource_registry, allocator)?;
        pass_profiler.prepare_finish(&self.pass);
        
        declaration.apply(
            resource_state_tracker,
            &|image| resource_registry.get_physical_image(image),
            &|buffer| resource_registry.get_physical_buffer(buffer),
        );
        resource_state_tracker.flush(&pass_context);

        pass_profiler.record_commands_start(&self.pass);
        pass_profiler.dispatch_start(&self.pass, &pass_context);
        self.pass.record_commands(&pass_context, &resource_registry, data)?;
        pass_profiler.dispatch_finish(&self.pass, &pass_context);
        pass_profiler.record_commands_finish(&self.pass);

        resource_state_tracker.flush(&pass_context);

        Ok(())
    }

    fn destroy(self: Box<Self>, resource_factories: &ResourceFactories) -> Result<()> {
        self.pass.destroy(&resource_factories)
    }
}
