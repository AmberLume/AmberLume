use crate::render::pass::pass_context::PassContext;
use anyhow::Result;
use crate::ids::FrameIndex;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::render_graph::virtual_image::render_targets::RenderTargets;
use crate::render::render_graph::resource_registry::resource_registry::ResourceRegistry;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;

pub trait Pass {
    type PassData;
    type Statistics;

    fn name(&self) -> String;

    fn is_enabled(&self) -> bool;

    fn prepare_data(
        &self,
        context: &FrameDataContext,
        resource_registry: &mut ResourceRegistry,
        allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData>;

    fn declare_resources(&self, _declaration: &mut PassResourceDeclaration) { }

    fn render_targets(&self) -> Option<RenderTargets> { None }

    fn record_commands(
        &self, 
        render_pass_context: &PassContext, 
        resource_registry: &ResourceRegistry, 
        data: Self::PassData,
    ) -> Result<()>;

    fn statistics(&self, frame_index: FrameIndex) -> Self::Statistics;
    
    fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> where Self: Sized;
}
