use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::image::managed_image_factory::ManagedImageFactory;
use crate::render::render_graph::resource_registry::resource_registry::ResourceRegistry;
use crate::render::render_graph::resource_state_tracker::resource_state_tracker::ResourceStateTracker;
use anyhow::Result;

pub struct PassGraphState {
    pub resource_registry: ResourceRegistry,
    pub resource_state_tracker: ResourceStateTracker,
}

impl PassGraphState {
    pub fn new() -> Self {
        Self {
            resource_registry: ResourceRegistry::new(),
            resource_state_tracker: ResourceStateTracker::new(),
        }
    }

    pub fn destroy(
        self,
        image_factory: &ManagedImageFactory,
        buffer_factory: &ManagedBufferFactory,
    ) -> Result<()> {
        self.resource_registry.destroy(image_factory, buffer_factory)
    }
}
