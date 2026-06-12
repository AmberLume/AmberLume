use crate::render::pass::pass_context::PassContext;
use crate::render::readback::readbacks::ReadbackSlice;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;

pub trait GpuReadback: Send + Sync {
    fn size(&self) -> u32;

    fn declare(&self, _declaration: &mut PassResourceDeclaration) { }

    fn record(&self, context: &PassContext, image_scope: &ImageResourceScope, buffer_scope: &BufferResourceScope, slice: &ReadbackSlice);

    fn sync(&self, slice: &ReadbackSlice);
}
