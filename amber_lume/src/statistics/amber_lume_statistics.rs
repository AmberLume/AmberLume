use std::sync::Arc;

use crate::profiler::frame_profile::FrameProfile;
use crate::render::renderer_statistics::RenderStatistics;
use crate::resources::store::providers_statistics::ResourcesStatistics;
use crate::ui::ui_statistics::UiStatistics;

pub struct AmberLumeStatistics {
    pub frame_profile: Arc<FrameProfile>,
    pub resources: ResourcesStatistics,
    pub render: RenderStatistics,
    pub ui: UiStatistics,
    pub ray_tracing_supported: bool,
}
