use gpu::profile_cpu_zone;
use gpu::profile_gpu_zone;
use gpu::FrameProfiler;
use gpu::ResourceFactories;
use crate::frame_context::FrameContext;
use crate::pass::Pass;
use crate::pass_entry::pass_entry::PassEntry;
use crate::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::resource_scope::image_resource_scope::ImageResourceScope;
use crate::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::resource_scope::readback_scope::ReadbackScope;
use crate::resource_scope::data_resource_scope::DataResourceScope;
use crate::virtual_buffer::heap_allocator::HeapAllocator;
use crate::virtual_image::render_targets::render_targets::RenderTargets;
use crate::virtual_image::resolved_render_targets::ResolvedRenderTargets;
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
    fn is_enabled(&self, data_scope: &DataResourceScope) -> bool {
        self.pass.is_enabled(data_scope)
    }

    fn render_targets(&self) -> Option<RenderTargets> {
        self.pass.render_targets()
    }

    fn declare_and_prepare(
        &mut self,
        declaration: &mut PassResourceDeclaration,
        data_scope: &mut DataResourceScope,
        buffer_scope: &mut BufferResourceScope,
        profiler: &FrameProfiler,
        allocator: &mut HeapAllocator,
        frame_context: &FrameContext,
    ) -> Result<()> {
        declaration.clear();
        self.pass.declare_resources(declaration);

        let data = profile_cpu_zone!(profiler, self.prepare_zone, {
            self.pass.prepare_data(data_scope, buffer_scope, allocator, frame_context)?
        });

        self.data = Some(data);

        Ok(())
    }

    fn record(
        &mut self,
        pass_context: &FrameContext,
        image_scope: &ImageResourceScope,
        buffer_scope: &BufferResourceScope,
        readback_scope: &ReadbackScope,
        profiler: &FrameProfiler,
        render_targets: Option<ResolvedRenderTargets>,
    ) -> Result<()> {
        let data = self.data.take().expect("declare_and_prepare must run before record");
        let command_buffer = pass_context.command_buffer();

        profile_cpu_zone!(profiler, self.record_zone, {
            profile_gpu_zone!(profiler, command_buffer, self.dispatch_zone, {
                if let Some(render_targets) = &render_targets {
                    render_targets.open(pass_context);
                }

                self.pass.record_commands(pass_context, image_scope, buffer_scope, readback_scope, data)?;

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
