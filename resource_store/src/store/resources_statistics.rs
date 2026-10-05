use crate::store::animation::backend::animation_backend::AnimationBackend;
use crate::store::image::backend::image_backend::ImageBackend;
use crate::store::material::backend::material_backend::MaterialBackend;
use crate::store::mesh::backend::mesh_backend::MeshBackend;
use resource_residency::ResourceBackend;
use resource_residency::ResourceUsageStatistics;
use crate::store::skeleton::backend::skeleton_backend::SkeletonBackend;

pub struct ResourcesStatistics {
    pub image_provider: ResourceUsageStatistics<<ImageBackend as ResourceBackend>::Statistics>,
    pub skeleton_provider: ResourceUsageStatistics<<SkeletonBackend as ResourceBackend>::Statistics>,
    pub animation_provider: ResourceUsageStatistics<<AnimationBackend as ResourceBackend>::Statistics>,
    pub material_provider: ResourceUsageStatistics<<MaterialBackend as ResourceBackend>::Statistics>,
    pub mesh_provider: ResourceUsageStatistics<<MeshBackend as ResourceBackend>::Statistics>,
}
