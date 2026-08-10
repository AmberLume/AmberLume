use std::sync::Arc;
use anyhow::Result;
use tracing::info;
use gpu::ResourceFactories;
use render_graph::FrameContext;
use crate::render::readback::readbacks::Readbacks;
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
use render_graph::ImageResourceScope;
use render_graph::BufferResourceScope;
use render_graph::DataResourceScope;
use render_graph::HeapAllocator;

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

    fn is_enabled(&self, _data_scope: &DataResourceScope) -> bool {
        true
    }

    fn prepare_data(
        &self,
        _data_scope: &mut DataResourceScope,
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
        context: &FrameContext,
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
