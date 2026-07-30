use gpu::ManagedBufferFactory;
use gpu::ManagedImageFactory;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::render_graph::resource_state_tracker::resource_state_tracker::ResourceStateTracker;
use anyhow::Result;

pub struct PassGraphState {
    pub image_scope: ImageResourceScope,
    pub buffer_scope: BufferResourceScope,
    pub resource_state_tracker: ResourceStateTracker,
}

impl PassGraphState {
    pub fn new() -> Self {
        Self {
            image_scope: ImageResourceScope::new(),
            buffer_scope: BufferResourceScope::new(),
            resource_state_tracker: ResourceStateTracker::new(),
        }
    }

    pub fn destroy(
        self,
        image_factory: &ManagedImageFactory,
        buffer_factory: &ManagedBufferFactory,
    ) -> Result<()> {
        self.image_scope.destroy(image_factory)?;
        self.buffer_scope.destroy(buffer_factory)?;

        Ok(())
    }
}
