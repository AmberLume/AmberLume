use std::sync::Arc;
use gpu::FrameProfile;
use pipeline_store::PipelineStatistics;
use resource_store::ResourcesStatistics;
use crate::render_statistics::RenderStatistics;
use crate::ui_statistics::UiStatistics;

pub struct AmberLumeStatistics {
    pub frame_profile: Arc<FrameProfile>,
    pub resources: ResourcesStatistics,
    pub pipelines: PipelineStatistics,
    pub render: RenderStatistics,
    pub ui: UiStatistics,
    pub ray_tracing_supported: bool,
}
