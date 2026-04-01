use crate::render::renderer_statistics::RenderStatistics;
use crate::resources::resource_hub_statistics::ResourcesStatistics;

pub struct AmberLumeStatistics {
    pub resources: ResourcesStatistics,
    pub render: RenderStatistics,
}
