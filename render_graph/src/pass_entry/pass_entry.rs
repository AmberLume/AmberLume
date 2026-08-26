use gpu::FrameProfiler;
use gpu::ResourceFactories;
use crate::frame_context::FrameContext;
use crate::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::resource_scope::data_resource_scope::DataResourceScope;
use crate::resource_scope::prepare_scopes::PrepareScopes;
use crate::resource_scope::record_scopes::RecordScopes;
use crate::virtual_image::render_targets::render_targets::RenderTargets;
use crate::virtual_image::resolved_render_targets::ResolvedRenderTargets;
use anyhow::Result;

pub trait PassEntry {
    fn name(&self) -> &'static str;

    fn is_enabled(&self, data_scope: &DataResourceScope) -> bool;

    fn render_targets(&self) -> Option<RenderTargets>;

    fn declare_and_prepare(
        &mut self,
        declaration: &mut PassResourceDeclaration,
        scopes: &mut PrepareScopes,
        profiler: &FrameProfiler,
        frame_context: &FrameContext,
    ) -> Result<()>;

    fn record(
        &mut self,
        pass_context: &FrameContext,
        scopes: &RecordScopes,
        profiler: &FrameProfiler,
        render_targets: Option<ResolvedRenderTargets>,
    ) -> Result<()>;

    fn destroy(self: Box<Self>, resource_factories: &ResourceFactories) -> Result<()>;
}
