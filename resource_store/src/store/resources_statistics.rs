use crate::store::providers::animation::animation_backend::AnimationBackend;
use crate::store::providers::image::image_backend::ImageBackend;
use crate::store::providers::material::material_backend::MaterialBackend;
use crate::store::providers::mesh::mesh_backend::MeshBackend;
use resource_residency::ResourceBackend;
use resource_residency::ResourceUsageStatistics;
use crate::store::providers::skeleton::skeleton_backend::SkeletonBackend;

pub struct ResourcesStatistics {
    pub image_provider: ResourceUsageStatistics<<ImageBackend as ResourceBackend>::Statistics>,
    pub skeleton_provider: ResourceUsageStatistics<<SkeletonBackend as ResourceBackend>::Statistics>,
    pub animation_provider: ResourceUsageStatistics<<AnimationBackend as ResourceBackend>::Statistics>,
    pub material_provider: ResourceUsageStatistics<<MaterialBackend as ResourceBackend>::Statistics>,
    pub mesh_provider: ResourceUsageStatistics<<MeshBackend as ResourceBackend>::Statistics>,
}
