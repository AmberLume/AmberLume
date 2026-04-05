use crate::resources::dynamic::animation::animation_backend::AnimationBackend;
use crate::resources::dynamic::mesh::mesh_backend::MeshBackend;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::resource_hub::ResourceHub;
use shipyard::Unique;
use std::sync::Arc;

#[derive(Unique)]
pub struct ResourceResolverUnique {
    pub mesh_provider: Arc<ResourceProvider<MeshBackend>>,
    pub animation_provider: Arc<ResourceProvider<AnimationBackend>>,
}

impl ResourceResolverUnique {
    pub fn new(resource_hub: Arc<ResourceHub>) -> Self {
        Self {
            mesh_provider: resource_hub.mesh_provider.clone(),
            animation_provider: resource_hub.animation_provider.clone(),
        }
    }
}
