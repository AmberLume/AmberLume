use crate::render::factories::image::managed_image_factory::ManagedImageFactory;
use crate::render::render_graph::resource_registry::resource_registry::ResourceRegistry;
use anyhow::Result;

pub struct PassGraphState {
    pub resource_registry: ResourceRegistry,
}

impl PassGraphState {
    pub fn new() -> Self {
        Self {
            resource_registry: ResourceRegistry::new(),
        }
    }

    pub fn destroy(self, image_factory: &ManagedImageFactory) -> Result<()> {
        self.resource_registry.destroy(image_factory)
    }
}
