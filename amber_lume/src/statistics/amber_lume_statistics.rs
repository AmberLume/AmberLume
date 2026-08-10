use std::sync::Arc;

use gpu::FrameProfile;
use crate::render::renderer_statistics::RenderStatistics;
use resource_store::ResourcesStatistics;
use pipeline_store::PipelineStatistics;
use crate::ui::ui_statistics::UiStatistics;

pub struct AmberLumeStatistics {
    pub frame_profile: Arc<FrameProfile>,
    pub resources: ResourcesStatistics,
    pub pipelines: PipelineStatistics,
    pub render: RenderStatistics,
    pub ui: UiStatistics,
    pub ray_tracing_supported: bool,
}
