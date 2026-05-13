use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_graph::pass_entry::pass_entry::PassEntry;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use anyhow::Result;
use crate::render::render_graph::virtual_image::render_targets::RenderTargets;
use crate::render::render_graph::resource_registry::resource_registry::ResourceRegistry;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_image::resolved_render_targets::ResolvedRenderTargets;
use crate::render::statistics::pass_profiler::PassProfiler;

pub struct ConcretePassEntry<P: Pass> {
    pub pass: P,
    data: Option<P::PassData>,
}

impl<P: Pass> ConcretePassEntry<P> {
    pub fn new(pass: P) -> Self {
        Self { pass, data: None }
    }
}

impl<P: Pass> PassEntry for ConcretePassEntry<P> {
    fn is_enabled(&self) -> bool {
        self.pass.is_enabled()
    }

    fn render_targets(&self) -> Option<RenderTargets> {
        self.pass.render_targets()
    }

    fn declare_and_prepare(
        &mut self,
        frame_data_context: &FrameDataContext,
        declaration: &mut PassResourceDeclaration,
        resource_registry: &mut ResourceRegistry,
        profiler: &mut PassProfiler,
        allocator: &mut HeapAllocator,
    ) -> Result<()> {
        declaration.clear();
        self.pass.declare_resources(declaration);

        profiler.prepare_start(&self.pass);
        let data = self.pass.prepare_data(frame_data_context, resource_registry, allocator)?;
        profiler.prepare_finish(&self.pass);

        self.data = Some(data);

        Ok(())
    }

    fn record(
        &mut self,
        pass_context: &PassContext,
        resource_registry: &ResourceRegistry,
        profiler: &mut PassProfiler,
        render_targets: Option<ResolvedRenderTargets>,
    ) -> Result<()> {
        let data = self.data.take().expect("declare_and_prepare must run before record");

        profiler.record_commands_start(&self.pass);
        profiler.dispatch_start(&self.pass, pass_context);

        if let Some(render_targets) = &render_targets {
            render_targets.open(pass_context);
        }

        self.pass.record_commands(pass_context, resource_registry, data)?;

        if render_targets.is_some() {
            pass_context.end_rendering();
        }

        profiler.dispatch_finish(&self.pass, pass_context);
        profiler.record_commands_finish(&self.pass);

        Ok(())
    }

    fn destroy(self: Box<Self>, resource_factories: &ResourceFactories) -> Result<()> {
        self.pass.destroy(&resource_factories)
    }
}
