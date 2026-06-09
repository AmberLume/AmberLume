use crate::render::pass::pass_context::PassContext;
use crate::render::readback::readbacks::ReadbackSlice;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::render_graph::resource_registry::resource_registry::ResourceRegistry;

pub trait GpuReadback: Send + Sync {
    fn size(&self) -> u32;

    fn declare(&self, _declaration: &mut PassResourceDeclaration) { }

    fn record(&self, context: &PassContext, resource_registry: &ResourceRegistry, slice: &ReadbackSlice);

    fn sync(&self, slice: &ReadbackSlice);
}
