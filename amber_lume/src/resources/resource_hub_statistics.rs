use crate::resources::dynamic::resource_usage_statistics::ResourceUsageStatistics;

pub struct ResourcesStatistics {
    pub image_provider: ResourceUsageStatistics,
    pub skeleton_provider: ResourceUsageStatistics,
    pub material_provider: ResourceUsageStatistics,
    pub mesh_provider: ResourceUsageStatistics,
    pub pipeline_provider: ResourceUsageStatistics,
    pub compute_pipeline_provider: ResourceUsageStatistics,
}
