use render_graph::FrameContext;
use crate::render::readback::readbacks::ReadbackSlice;
use render_graph::PassResourceDeclaration;
use render_graph::ImageResourceScope;
use render_graph::BufferResourceScope;

pub trait GpuReadback: Send + Sync {
    fn size(&self) -> u32;

    fn declare(&self, _declaration: &mut PassResourceDeclaration) { }

    fn record(&self, context: &FrameContext, image_scope: &ImageResourceScope, buffer_scope: &BufferResourceScope, slice: &ReadbackSlice);

    fn sync(&self, slice: &ReadbackSlice);
}
