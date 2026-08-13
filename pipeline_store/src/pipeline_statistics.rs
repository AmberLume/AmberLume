use resource_residency::ResourceBackend;
use resource_residency::ResourceUsageStatistics;
use crate::compute_pipeline_backend::ComputePipelineBackend;
use crate::pipeline_backend::PipelineBackend;

pub struct PipelineStatistics {
    pub pipeline_provider: ResourceUsageStatistics<<PipelineBackend as ResourceBackend>::Statistics>,
    pub compute_pipeline_provider: ResourceUsageStatistics<<ComputePipelineBackend as ResourceBackend>::Statistics>,
}
