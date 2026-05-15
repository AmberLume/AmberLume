use crate::profile_cpu_zone;
use crate::profile_gpu_zone;
use crate::profiler::frame_profiler::FrameProfiler;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_graph::pass::Pass;
use crate::render::render_graph::pass_entry::pass_entry::PassEntry;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::render_graph::resource_registry::resource_registry::ResourceRegistry;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_image::render_targets::RenderTargets;
use crate::render::render_graph::virtual_image::resolved_render_targets::ResolvedRenderTargets;
use anyhow::Result;

pub struct ConcretePassEntry<P: Pass> {
    pub pass: P,
    data: Option<P::PassData>,

    prepare_zone: &'static str,
    record_zone: &'static str,
    dispatch_zone: &'static str,
}

impl<P: Pass> ConcretePassEntry<P> {
    pub fn new(pass: P) -> Self {
        let name = pass.name();

        let prepare_zone = Box::leak(format!("{name}.prepare").into_boxed_str());
        let record_zone = Box::leak(format!("{name}.record").into_boxed_str());
        let dispatch_zone = Box::leak(format!("{name}.dispatch").into_boxed_str());

        Self {
            pass,
            data: None,
            prepare_zone,
            record_zone,
            dispatch_zone,
        }
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
        profiler: &FrameProfiler,
        allocator: &mut HeapAllocator,
    ) -> Result<()> {
        declaration.clear();
        self.pass.declare_resources(declaration);

        let data = profile_cpu_zone!(profiler, self.prepare_zone, {
            self.pass.prepare_data(frame_data_context, resource_registry, allocator)?
        });

        self.data = Some(data);

        Ok(())
    }

    fn record(
        &mut self,
        pass_context: &PassContext,
        resource_registry: &ResourceRegistry,
        profiler: &FrameProfiler,
        render_targets: Option<ResolvedRenderTargets>,
    ) -> Result<()> {
        let data = self.data.take().expect("declare_and_prepare must run before record");
        let command_buffer = pass_context.command_recording.command_buffer;

        profile_cpu_zone!(profiler, self.record_zone, {
            profile_gpu_zone!(profiler, command_buffer, self.dispatch_zone, {
                if let Some(render_targets) = &render_targets {
                    render_targets.open(pass_context);
                }

                self.pass.record_commands(pass_context, resource_registry, data)?;

                if render_targets.is_some() {
                    pass_context.end_rendering();
                }
            });
        });

        Ok(())
    }

    fn destroy(self: Box<Self>, resource_factories: &ResourceFactories) -> Result<()> {
        self.pass.destroy(&resource_factories)
    }
}
