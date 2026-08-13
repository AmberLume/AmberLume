use crate::render::render_context::RenderContext;
use gpu::PipelineLayoutRegistry;
use pipeline_store::ComputePipelineBackend;
use pipeline_store::PipelineBackend;
use resource_residency::ResourceProvider;
use resource_store::ResourceBuffers;

pub struct PassResources<'a> {
    pub render_context: &'a RenderContext,
    pub resource_buffers: &'a ResourceBuffers,
    pub pipeline_provider: &'a ResourceProvider<PipelineBackend>,
    pub compute_pipeline_provider: &'a ResourceProvider<ComputePipelineBackend>,
    pub pipeline_layout_registry: &'a PipelineLayoutRegistry,
}
