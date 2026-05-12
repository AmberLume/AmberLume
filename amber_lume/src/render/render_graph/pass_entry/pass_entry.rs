use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use anyhow::Result;
use crate::render::render_graph::virtual_image::render_targets::RenderTargets;
use crate::render::render_graph::resource_registry::resource_registry::ResourceRegistry;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_image::resolved_render_targets::ResolvedRenderTargets;
use crate::render::statistics::pass_profiler::PassProfiler;

pub trait PassEntry {
    fn is_enabled(&self) -> bool;

    fn render_targets(&self) -> Option<RenderTargets>;

    fn declare_and_prepare(
        &mut self,
        frame_data_context: &FrameDataContext,
        declaration: &mut PassResourceDeclaration,
        resource_registry: &mut ResourceRegistry,
        profiler: &mut PassProfiler,
        allocator: &mut HeapAllocator,
    ) -> Result<()>;

    fn record(
        &mut self,
        pass_context: &PassContext,
        resource_registry: &ResourceRegistry,
        profiler: &mut PassProfiler,
        render_targets: Option<ResolvedRenderTargets>,
    ) -> Result<()>;

    fn destroy(self: Box<Self>, resource_factories: &ResourceFactories) -> Result<()>;
}
