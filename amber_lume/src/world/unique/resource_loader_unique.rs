use std::sync::Arc;
use shipyard::Unique;
use crate::resources::index::resource_index::ResourceIndex;

#[derive(Unique)]
pub struct ResourceLoaderUnique {
    pub resource_loader: Arc<ResourceIndex>,
}

impl ResourceLoaderUnique {
    pub fn new(resource_index: Arc<ResourceIndex>) -> Self {
        Self {
            resource_loader: resource_index,
        }
    }
}
