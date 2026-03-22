use crate::resources::dynamic::mesh::mesh_backend::MeshBackend;
use crate::resources::resource_hub::ResourceHub;
use shipyard::Unique;
use std::sync::Arc;
use crate::resources::dynamic::resource_provider::ResourceProvider;

#[derive(Unique)]
pub struct ResourceResolverUnique {
    pub model_provider: Arc<ResourceProvider<MeshBackend>>,
}

impl ResourceResolverUnique {
    pub fn new(resource_hub: Arc<ResourceHub>) -> Self {
        Self {
            model_provider: resource_hub.get_model_provider(),
        }
    }
}
