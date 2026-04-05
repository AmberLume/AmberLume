use crate::resources::dynamic::animation::animation_backend::AnimationBackend;
use crate::resources::dynamic::compute_pipeline::compute_pipeline_backend::ComputePipelineBackend;
use crate::resources::dynamic::image::image_backend::ImageBackend;
use crate::resources::dynamic::material::material_backend::MaterialBackend;
use crate::resources::dynamic::mesh::mesh_backend::MeshBackend;
use crate::resources::dynamic::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::dynamic::resource_backend::ResourceBackend;
use crate::resources::dynamic::resource_usage_statistics::ResourceUsageStatistics;
use crate::resources::dynamic::skeleton::skeleton_backend::SkeletonBackend;

pub struct ResourcesStatistics {
    pub image_provider: ResourceUsageStatistics<<ImageBackend as ResourceBackend>::Statistics>,
    pub skeleton_provider: ResourceUsageStatistics<<SkeletonBackend as ResourceBackend>::Statistics>,
    pub animation_provider: ResourceUsageStatistics<<AnimationBackend as ResourceBackend>::Statistics>,
    pub material_provider: ResourceUsageStatistics<<MaterialBackend as ResourceBackend>::Statistics>,
    pub mesh_provider: ResourceUsageStatistics<<MeshBackend as ResourceBackend>::Statistics>,
    pub pipeline_provider: ResourceUsageStatistics<<PipelineBackend as ResourceBackend>::Statistics>,
    pub compute_pipeline_provider: ResourceUsageStatistics<<ComputePipelineBackend as ResourceBackend>::Statistics>,
}
