use render_graph::Pass;
use render_graph::FrameContext;
use crate::render::pass::pass_resources::PassResources;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, CompareOp, Format, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::sync::Arc;
use tracing::info;
use gpu::ResourceFactories;
use crate::render::pass::shadows::cascade_shadows::cascade_shadows_push_constants::CascadeShadowsPushConstants;
use render_graph::PassResourceDeclaration;
use render_graph::PrepareScopes;
use render_graph::RecordScopes;
use render_graph::DataResourceScope;
use render_graph::DrawBucket;
use crate::render::pass::draw_pool::DrawPool;
use render_graph::VirtualBuffer;
use render_graph::{DepthTarget, RenderTargets};
use render_graph::VirtualImage;
use gpu::PipelineLayoutType;
use pipeline_store::PipelineConfig;
use pipeline_store::PipelineStageConfig;
use resource_residency::ResRef;
use crate::resource_manifest::shaders;

pub struct CascadeShadowsPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    shadows_image: VirtualImage,
    view_mask: u32,

    entity_buffer: VirtualBuffer,
    shadow_cascades_buffer: VirtualBuffer,
    pool: DrawPool,
    bucket: DrawBucket,
    bone_transform: VirtualBuffer,

    mesh_vertex_buffer: VirtualBuffer,

    mesh_vertex_skin_buffer: VirtualBuffer,

    submesh_buffer: VirtualBuffer,
    index_buffer: VirtualBuffer,
}

impl CascadeShadowsPass {
    pub fn create(
        resources: &PassResources,
        cascade_count: u32,
        depth_format: Format,
        shadows_image: VirtualImage,
        entity_buffer: VirtualBuffer,
        shadow_cascades_buffer: VirtualBuffer,
        pool: DrawPool,
        bucket: DrawBucket,
        bone_transform: VirtualBuffer,
    ) -> Result<Self> {
        let view_mask = (1u32 << cascade_count) - 1;

        let pipeline_config = PipelineConfig {
            label: String::from("cascade_shadows"),
            stages: vec![
                PipelineStageConfig::vertex(shaders::SHADOWS_VERT),
            ],
            color_formats: vec![],
            depth_format: Some(depth_format),
            view_mask,
            depth_bias_enable: true,
            depth_bias_constant_factor: 1.5,
            depth_bias_slope_factor: 2.0,
            depth_compare_op: CompareOp::LESS_OR_EQUAL,
            ..PipelineConfig::geometry()
        };

        let _handle = resources.pipeline_provider.acquire_sync(pipeline_config)?;
        let Some(pipeline) = resources.pipeline_provider.with_resource(_handle.id, |pipeline| *pipeline) else {
            bail!("Failed to acquire Pipeline");
        };

        Ok(Self {
            _handle,

            pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            shadows_image,
            view_mask,

            entity_buffer,
            shadow_cascades_buffer,
            pool,
            bucket,
            bone_transform,

            mesh_vertex_buffer: resources.resource_buffer_handles.mesh_vertex_buffer,

            mesh_vertex_skin_buffer: resources.resource_buffer_handles.mesh_vertex_skin_buffer,

            submesh_buffer: resources.resource_buffer_handles.submesh_buffer,
            index_buffer: resources.resource_buffer_handles.index_buffer,
        })
    }
}

impl Pass for CascadeShadowsPass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("cascade_shadows")
    }

    fn is_enabled(&self, _data_scope: &DataResourceScope) -> bool {
        true
    }

    fn prepare_data(
        &self,
        _scopes: &mut PrepareScopes,
        _frame_context: &FrameContext,
    ) -> Result<Self::PassData> {
        Ok(())
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .write_image(
                self.shadows_image,
                ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                PipelineStageFlags::EARLY_FRAGMENT_TESTS | PipelineStageFlags::LATE_FRAGMENT_TESTS,
            )
            .read_buffer(
                self.entity_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER | PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_buffer(
                self.shadow_cascades_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER,
            )
            .read_buffer(
                self.pool.draw_count,
                AccessFlags::INDIRECT_COMMAND_READ,
                PipelineStageFlags::DRAW_INDIRECT,
            )
            .read_buffer(
                self.pool.indirect,
                AccessFlags::INDIRECT_COMMAND_READ,
                PipelineStageFlags::DRAW_INDIRECT,
            )
            .read_buffer(
                self.pool.draw_data,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER,
            )
            .read_buffer(
                self.bone_transform,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER,
            )
            .read_buffer(
                self.index_buffer,
                AccessFlags::INDEX_READ,
                PipelineStageFlags::VERTEX_INPUT,
            )
            .read_buffer(
                self.mesh_vertex_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER | PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_buffer(
                self.mesh_vertex_skin_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER | PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_buffer(
                self.submesh_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER | PipelineStageFlags::FRAGMENT_SHADER,
            );
    }

    fn render_targets(&self) -> Option<RenderTargets> {
        Some(RenderTargets {
            color: Vec::new(),
            depth: Some(DepthTarget { image: self.shadows_image, clear: Some(1.0) }),
            view_mask: self.view_mask,
        })
    }

    fn record_commands(
        &self,
        context: &FrameContext,
        scopes: &RecordScopes,
        _data: Self::PassData,
    ) -> Result<()> {
        let index_buffer = scopes.buffer.get_physical_buffer(self.index_buffer);
        let mesh_vertex_buffer = scopes.buffer.get_physical_buffer(self.mesh_vertex_buffer);
        let mesh_vertex_skin_buffer = scopes.buffer.get_physical_buffer(self.mesh_vertex_skin_buffer);
        let submesh_buffer = scopes.buffer.get_physical_buffer(self.submesh_buffer);

        let entity_buffer = scopes.buffer.get_physical_buffer(self.entity_buffer);
        let shadow_cascades_buffer = scopes.buffer.get_physical_buffer(self.shadow_cascades_buffer);
        let draw_count = scopes.buffer.get_physical_buffer(self.pool.draw_count);
        let indirect = scopes.buffer.get_physical_buffer(self.pool.indirect);
        let draw_data = scopes.buffer.get_physical_buffer(self.pool.draw_data);
        let bone_transform_buffer = scopes.buffer.get_physical_buffer(self.bone_transform);

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.bind_index_buffer(index_buffer.range);

        context.push_constants(
            self.pipeline_layout,
            &CascadeShadowsPushConstants::create(
                draw_data.range,
                entity_buffer.range,
                submesh_buffer.range,
                mesh_vertex_buffer.range,
                mesh_vertex_skin_buffer.range,
                bone_transform_buffer.range,
                shadow_cascades_buffer.range,
            ),
        );
        context.draw_indirect_gpu_scene(&indirect, &draw_count, self.bucket);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("CascadeShadowsPass destroyed");

        Ok(())
    }
}
