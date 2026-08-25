use crate::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::resource_scope::data_resource_scope::DataResourceScope;
use crate::resource_scope::image_resource_scope::ImageResourceScope;
use crate::resource_scope::readback_scope::ReadbackScope;
use crate::resource_state_tracker::resource_state_tracker::ResourceStateTracker;
use index_allocator::ResourceLimits;
use anyhow::Result;
use gpu::ManagedBufferFactory;
use gpu::ManagedImageFactory;
use gpu::ResourceFactories;
use std::sync::Arc;

pub struct PassGraphState {
    pub image_scope: ImageResourceScope,
    pub buffer_scope: BufferResourceScope,
    pub data_scope: DataResourceScope,
    pub readback_scope: ReadbackScope,
    pub resource_state_tracker: ResourceStateTracker,
}

impl PassGraphState {
    pub fn create(
        resource_factories: Arc<ResourceFactories>,
        limits: ResourceLimits,
        frame_count: u32,
        ray_tracing: bool,
    ) -> Result<Self> {
        Ok(Self {
            image_scope: ImageResourceScope::new(),
            buffer_scope: BufferResourceScope::create(resource_factories, limits, frame_count, ray_tracing)?,
            data_scope: DataResourceScope::new(),
            readback_scope: ReadbackScope::new(),
            resource_state_tracker: ResourceStateTracker::new(),
        })
    }

    pub fn destroy(
        self,
        image_factory: &ManagedImageFactory,
        buffer_factory: &ManagedBufferFactory,
    ) -> Result<()> {
        self.image_scope.destroy(image_factory)?;
        self.buffer_scope.destroy()?;
        self.readback_scope.destroy(buffer_factory)?;

        Ok(())
    }
}
