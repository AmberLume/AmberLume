use crate::profiler::frame_profiler::FrameProfiler;
use crate::render::pass::pass_context::PassContext;
use anyhow::Result;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::render_graph::virtual_image::render_targets::RenderTargets;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;

pub trait Pass {
    type PassData;

    fn name(&self) -> String;

    fn is_enabled(&self) -> bool;

    fn prepare_data(
        &self,
        context: &FrameDataContext,
        buffer_scope: &mut BufferResourceScope,
        allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData>;

    fn declare_resources(&self, _declaration: &mut PassResourceDeclaration) { }

    fn render_targets(&self) -> Option<RenderTargets> { None }

    fn record_commands(
        &self,
        render_pass_context: &PassContext,
        image_scope: &ImageResourceScope,
        buffer_scope: &BufferResourceScope,
        data: Self::PassData,
    ) -> Result<()>;

    fn register_with_profiler(&self, _profiler: &FrameProfiler) { }

    fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> where Self: Sized;
}
