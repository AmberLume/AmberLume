use crate::resources::store::providers::animation::animation_backend::AnimationBackend;
use crate::resources::store::providers::compute_pipeline::compute_pipeline_backend::ComputePipelineBackend;
use crate::resources::store::providers::image::image_backend::ImageBackend;
use crate::resources::store::providers::material::material_backend::MaterialBackend;
use crate::resources::store::providers::mesh::mesh_backend::MeshBackend;
use crate::resources::store::providers::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::store::providers::resource_backend::ResourceBackend;
use crate::resources::store::providers::resource_usage_statistics::ResourceUsageStatistics;
use crate::resources::store::providers::skeleton::skeleton_backend::SkeletonBackend;

pub struct ResourcesStatistics {
    pub image_provider: ResourceUsageStatistics<<ImageBackend as ResourceBackend>::Statistics>,
    pub skeleton_provider: ResourceUsageStatistics<<SkeletonBackend as ResourceBackend>::Statistics>,
    pub animation_provider: ResourceUsageStatistics<<AnimationBackend as ResourceBackend>::Statistics>,
    pub material_provider: ResourceUsageStatistics<<MaterialBackend as ResourceBackend>::Statistics>,
    pub mesh_provider: ResourceUsageStatistics<<MeshBackend as ResourceBackend>::Statistics>,
    pub pipeline_provider: ResourceUsageStatistics<<PipelineBackend as ResourceBackend>::Statistics>,
    pub compute_pipeline_provider: ResourceUsageStatistics<<ComputePipelineBackend as ResourceBackend>::Statistics>,
}
