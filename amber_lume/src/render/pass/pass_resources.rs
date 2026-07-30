use std::sync::Arc;
use arc_swap::ArcSwap;
use crate::render::render_context::RenderContext;
use gpu::PipelineLayoutRegistry;
use crate::resources::store::providers::compute_pipeline::compute_pipeline_backend::ComputePipelineBackend;
use crate::resources::store::providers::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::store::providers::resource_provider::ResourceProvider;
use crate::settings::settings::EngineSettings;

pub struct PassResources<'a> {
    pub render_context: &'a RenderContext,
    pub pipeline_provider: &'a ResourceProvider<PipelineBackend>,
    pub compute_pipeline_provider: &'a ResourceProvider<ComputePipelineBackend>,
    pub pipeline_layout_registry: &'a PipelineLayoutRegistry,
    pub settings: &'a Arc<ArcSwap<EngineSettings>>,
}
