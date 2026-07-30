use gpu::FrameProfiler;
use gpu::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_image::render_targets::RenderTargets;
use crate::render::render_graph::virtual_image::resolved_render_targets::ResolvedRenderTargets;
use anyhow::Result;

pub trait PassEntry {
    fn is_enabled(&self) -> bool;

    fn render_targets(&self) -> Option<RenderTargets>;

    fn declare_and_prepare(
        &mut self,
        frame_data_context: &FrameDataContext,
        declaration: &mut PassResourceDeclaration,
        buffer_scope: &mut BufferResourceScope,
        profiler: &FrameProfiler,
        allocator: &mut HeapAllocator,
    ) -> Result<()>;

    fn record(
        &mut self,
        pass_context: &PassContext,
        image_scope: &ImageResourceScope,
        buffer_scope: &BufferResourceScope,
        profiler: &FrameProfiler,
        render_targets: Option<ResolvedRenderTargets>,
    ) -> Result<()>;

    fn destroy(self: Box<Self>, resource_factories: &ResourceFactories) -> Result<()>;
}
