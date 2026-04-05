use crate::resources::dynamic::animation::animation_backend::AnimationBackend;
use crate::resources::dynamic::mesh::mesh_backend::MeshBackend;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::resource_hub::ResourceHub;
use shipyard::Unique;
use std::sync::Arc;
use crate::resources::dynamic::skinning::bone_transform_handler::BoneTransformHandler;

#[derive(Unique)]
pub struct ResourceResolverUnique {
    pub mesh_provider: Arc<ResourceProvider<MeshBackend>>,
    pub animation_provider: Arc<ResourceProvider<AnimationBackend>>,
    
    pub bone_transform_handler: Arc<BoneTransformHandler>,
}

impl ResourceResolverUnique {
    pub fn new(resource_hub: Arc<ResourceHub>) -> Self {
        Self {
            mesh_provider: resource_hub.mesh_provider.clone(),
            animation_provider: resource_hub.animation_provider.clone(),
            
            bone_transform_handler: resource_hub.bone_transform_handler.clone(),
        }
    }
}
