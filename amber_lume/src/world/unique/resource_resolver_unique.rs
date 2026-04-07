use crate::resources::store::providers::resource_provider::ResourceProvider;
use shipyard::Unique;
use std::sync::Arc;
use crate::resources::skinning::bone_transform_handler::BoneTransformHandler;
use crate::resources::store::providers::animation::animation_backend::AnimationBackend;
use crate::resources::store::providers::mesh::mesh_backend::MeshBackend;
use crate::resources::store::providers::skeleton::skeleton_backend::SkeletonBackend;
use crate::resources::store::resource_store::ResourceStore;

#[derive(Unique)]
pub struct ResourceResolverUnique {
    pub mesh_provider: Arc<ResourceProvider<MeshBackend>>,
    pub skeleton_provider: Arc<ResourceProvider<SkeletonBackend>>,
    pub animation_provider: Arc<ResourceProvider<AnimationBackend>>,
    
    pub bone_transform_handler: Arc<BoneTransformHandler>,
}

impl ResourceResolverUnique {
    pub fn new(
        resource_store: Arc<ResourceStore>,
        bone_transform_handler: Arc<BoneTransformHandler>,
    ) -> Self {
        Self {
            mesh_provider: resource_store.mesh_provider.clone(),
            animation_provider: resource_store.animation_provider.clone(),
            skeleton_provider: resource_store.skeletons_provider.clone(),

            bone_transform_handler: bone_transform_handler.clone(),
        }
    }
}
