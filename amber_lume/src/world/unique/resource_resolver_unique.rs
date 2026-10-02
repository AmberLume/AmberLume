use resource_residency::ResourceProvider;
use shipyard::Unique;
use std::sync::Arc;
use resource_store::AnimationBackend;
use resource_store::MeshBackend;
use resource_store::MeshLoader;
use resource_store::SkeletonBackend;
use resource_store::SkinLoader;
use resource_store::ResourceStore;

#[derive(Unique)]
pub struct ResourceResolverUnique {
    pub mesh_provider: Arc<ResourceProvider<MeshBackend>>,
    pub skeleton_provider: Arc<ResourceProvider<SkeletonBackend>>,
    pub animation_provider: Arc<ResourceProvider<AnimationBackend>>,

    pub mesh_loader: Arc<MeshLoader>,
    pub skin_loader: Arc<SkinLoader>,
}

impl ResourceResolverUnique {
    pub fn new(resource_store: Arc<ResourceStore>) -> Self {
        Self {
            mesh_provider: resource_store.mesh_provider.clone(),
            animation_provider: resource_store.animation_provider.clone(),
            skeleton_provider: resource_store.skeleton_provider.clone(),

            mesh_loader: resource_store.mesh_loader.clone(),
            skin_loader: resource_store.skin_loader.clone(),
        }
    }
}
