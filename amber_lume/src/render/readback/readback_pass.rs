use std::sync::Arc;
use anyhow::Result;
use tracing::info;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_context::PassContext;
use crate::render::readback::readbacks::Readbacks;
use crate::render::render_graph::pass::Pass;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;

pub struct ReadbackPass {
    readbacks: Arc<Readbacks>,
}

impl ReadbackPass {
    pub fn new(readbacks: Arc<Readbacks>) -> Self {
        Self {
            readbacks,
        }
    }
}

impl Pass for ReadbackPass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("readback")
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn prepare_data(
        &self,
        _context: &FrameDataContext,
        _buffer_scope: &mut BufferResourceScope,
        _allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        Ok(())
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        self.readbacks.declare(declaration);
    }

    fn record_commands(
        &self,
        context: &PassContext,
        image_scope: &ImageResourceScope,
        buffer_scope: &BufferResourceScope,
        _data: Self::PassData,
    ) -> Result<()> {
        self.readbacks.record(context, image_scope, buffer_scope);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("ReadbackPass destroyed");

        Ok(())
    }
}
