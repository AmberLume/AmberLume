use crate::render::renderer_statistics::RenderStatistics;
use crate::resources::resource_hub_statistics::ResourcesStatistics;
use crate::ui::ui_statistics::UiStatistics;

pub struct AmberLumeStatistics {
    pub resources: ResourcesStatistics,
    pub render: RenderStatistics,
    pub ui: UiStatistics,
}
