use gpu::FrameProfiler;
use crate::frame_context::FrameContext;
use anyhow::Result;
use gpu::ResourceFactories;
use crate::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::virtual_image::render_targets::RenderTargets;
use crate::resource_scope::image_resource_scope::ImageResourceScope;
use crate::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::resource_scope::readback_scope::ReadbackScope;
use crate::resource_scope::data_resource_scope::DataResourceScope;
use crate::virtual_buffer::heap_allocator::HeapAllocator;

pub trait Pass {
    type PassData;

    fn name(&self) -> String;

    fn is_enabled(&self, data_scope: &DataResourceScope) -> bool;

    fn prepare_data(
        &self,
        data_scope: &mut DataResourceScope,
        buffer_scope: &mut BufferResourceScope,
        allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData>;

    fn declare_resources(&self, _declaration: &mut PassResourceDeclaration) { }

    fn render_targets(&self) -> Option<RenderTargets> { None }

    fn record_commands(
        &self,
        frame_context: &FrameContext,
        image_scope: &ImageResourceScope,
        buffer_scope: &BufferResourceScope,
        readback_scope: &ReadbackScope,
        data: Self::PassData,
    ) -> Result<()>;

    fn register_with_profiler(&self, _profiler: &FrameProfiler) { }

    fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> where Self: Sized;
}
