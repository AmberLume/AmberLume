use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_graph::resource_state_tracker::resource_state_tracker::ResourceStateTracker;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use anyhow::Result;
use crate::render::render_graph::resource_registry::resource_registry::ResourceRegistry;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::statistics::pass_profiler::PassProfiler;

pub trait PassEntry {
    fn run(
        &mut self,
        frame_data_context: &FrameDataContext,
        pass_context: &PassContext,
        declaration: &mut PassResourceDeclaration,
        resource_state_tracker: &mut ResourceStateTracker,
        resource_registry: &mut ResourceRegistry,
        profiler: &mut PassProfiler,
        allocator: &mut HeapAllocator,
    ) -> Result<()>;

    fn destroy(self: Box<Self>, resource_factories: &ResourceFactories) -> Result<()>;
}
