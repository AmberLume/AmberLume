use gpu::ManagedBufferFactory;
use gpu::ManagedImageFactory;
use crate::resource_scope::image_resource_scope::ImageResourceScope;
use crate::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::resource_scope::data_resource_scope::DataResourceScope;
use crate::resource_scope::readback_scope::ReadbackScope;
use crate::resource_state_tracker::resource_state_tracker::ResourceStateTracker;
use anyhow::Result;

pub struct PassGraphState {
    pub image_scope: ImageResourceScope,
    pub buffer_scope: BufferResourceScope,
    pub data_scope: DataResourceScope,
    pub readback_scope: ReadbackScope,
    pub resource_state_tracker: ResourceStateTracker,
}

impl PassGraphState {
    pub fn new() -> Self {
        Self {
            image_scope: ImageResourceScope::new(),
            buffer_scope: BufferResourceScope::new(),
            data_scope: DataResourceScope::new(),
            readback_scope: ReadbackScope::new(),
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
        self.readback_scope.destroy(buffer_factory)?;

        Ok(())
    }
}
