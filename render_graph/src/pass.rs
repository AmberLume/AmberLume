use gpu::FrameProfiler;
use crate::frame_context::FrameContext;
use anyhow::Result;
use gpu::ResourceFactories;
use crate::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::virtual_image::render_targets::render_targets::RenderTargets;
use crate::resource_scope::data_resource_scope::DataResourceScope;
use crate::resource_scope::prepare_scopes::PrepareScopes;
use crate::resource_scope::record_scopes::RecordScopes;

pub trait Pass {
    type PassData;

    fn name(&self) -> String;

    fn is_enabled(&self, data_scope: &DataResourceScope) -> bool;

    fn prepare_data(
        &self,
        scopes: &mut PrepareScopes,
        frame_context: &FrameContext,
    ) -> Result<Self::PassData>;

    fn declare_resources(&self, _declaration: &mut PassResourceDeclaration) { }

    fn render_targets(&self) -> Option<RenderTargets> { None }

    fn record_commands(
        &self,
        frame_context: &FrameContext,
        scopes: &RecordScopes,
        data: Self::PassData,
    ) -> Result<()>;

    fn register_with_profiler(&self, _profiler: &FrameProfiler) { }

    fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> where Self: Sized;
}
