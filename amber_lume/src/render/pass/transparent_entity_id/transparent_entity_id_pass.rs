use gpu::ResourceFactories;
use render_graph::FrameContext;
use crate::render::pass_resources::pass_resources::PassResources;
use crate::render::pass::transparent_entity_id::transparent_entity_id_fragment_shader::TransparentEntityIdFragmentShader;
use crate::render::push_constants::pipeline_push_constants::PipelinePushConstants;
use crate::render::push_constants::push_constants::PushConstants;
use crate::render::push_constants::vertex::mesh_entity_velocity_vertex_shader::MeshEntityVelocityVertexShader;
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
use render_graph::DrawBucket;
use crate::render::draw_pool::draw_pool::DrawPool;
use render_graph::VirtualBuffer;
use render_graph::{ColorTarget, DepthTarget, RenderTargets};
use render_graph::VirtualImage;
use render_graph::PrepareScopes;
use render_graph::RecordScopes;
use render_graph::DataResourceScope;
use gpu::PipelineLayoutType;
use pipeline_store::PipelineConfig;
use resource_residency::ResRef;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, CompareOp, Format, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::sync::Arc;
use tracing::info;

pub struct TransparentEntityIdPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,
    push_constants: PipelinePushConstants<MeshEntityVelocityVertexShader, TransparentEntityIdFragmentShader>,

    entity_id_image: VirtualImage,
    velocity_image: VirtualImage,
    depth: VirtualImage,

    camera_buffer: VirtualBuffer,
    entity_buffer: VirtualBuffer,
    index_buffer: VirtualBuffer,
    vertex_position_buffer: VirtualBuffer,

    pool: DrawPool,
    bucket: DrawBucket,
}

impl TransparentEntityIdPass {
    pub fn create(
        resources: &PassResources,
        entity_id_image: VirtualImage,
        velocity_format: Format,
        velocity_image: VirtualImage,
        depth: VirtualImage,
        camera_buffer: VirtualBuffer,
        entity_buffer: VirtualBuffer,
        pool: DrawPool,
        bucket: DrawBucket,
    ) -> Result<Self> {
        let push_constants = PipelinePushConstants::<MeshEntityVelocityVertexShader, TransparentEntityIdFragmentShader>::new();

        let pipeline_config = PipelineConfig {
            label: "transparent_entity_id".to_string(),
            stages: push_constants.stages_layout(),
            color_formats: vec![Format::R32_UINT, velocity_format],
            depth_format: Some(resources.render_context.depth_format),
            depth_write: false,
            depth_compare_op: CompareOp::GREATER,
            ..PipelineConfig::geometry()
        };

        let _handle = resources.pipeline_provider.acquire_sync(pipeline_config)?;
        let Some(pipeline) = resources.pipeline_provider.with_resource(_handle.id, |pipeline| *pipeline) else {
            bail!("Failed to acquire Pipeline for transparent_entity_id");
        };

        Ok(Self {
            _handle,

            pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),
            push_constants,

            entity_id_image,
            velocity_image,
            depth,

            camera_buffer,
            entity_buffer,
            index_buffer: resources.resource_buffer_handles.index_buffer,
            vertex_position_buffer: resources.resource_buffer_handles.vertex_position_buffer,

            pool,
            bucket,
        })
    }
}

impl Pass for TransparentEntityIdPass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("transparent_entity_id")
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
                self.velocity_image,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_WRITE | AccessFlags::COLOR_ATTACHMENT_READ,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            )
            .write_image(
                self.entity_id_image,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_WRITE,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            )
            .read_image(
                self.depth,
                ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
                PipelineStageFlags::EARLY_FRAGMENT_TESTS | PipelineStageFlags::LATE_FRAGMENT_TESTS,
            )
            .read_buffer(
                self.camera_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER,
            )
            .read_buffer(
                self.entity_buffer,
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
                self.index_buffer,
                AccessFlags::INDEX_READ,
                PipelineStageFlags::VERTEX_INPUT,
            )
            .read_buffer(
                self.vertex_position_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER,
            );
    }

    fn render_targets(&self) -> Option<RenderTargets> {
        Some(RenderTargets {
            color: vec![
                ColorTarget {
                    image: self.entity_id_image,
                    mip: None,
                    clear: None,
                },
                ColorTarget {
                    image: self.velocity_image,
                    mip: None,
                    clear: None,
                },
            ],
            depth: Some(DepthTarget {
                image: self.depth,
                clear: None,
            }),
            view_mask: 0,
        })
    }

    fn record_commands(
        &self,
        context: &FrameContext,
        scopes: &RecordScopes,
        _data: Self::PassData,
    ) -> Result<()> {
        let index_buffer = scopes.buffer.get_physical_buffer(self.index_buffer);

        let camera_buffer = scopes.buffer.get_physical_buffer(self.camera_buffer);
        let entity_buffer = scopes.buffer.get_physical_buffer(self.entity_buffer);
        let vertex_position_buffer = scopes.buffer.get_physical_buffer(self.vertex_position_buffer);
        let draw_count = scopes.buffer.get_physical_buffer(self.pool.draw_count);
        let indirect = scopes.buffer.get_physical_buffer(self.pool.indirect);
        let draw_data = scopes.buffer.get_physical_buffer(self.pool.draw_data);

        context.bind_index_buffer(index_buffer.range);

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        self.push_constants.push(
            context,
            self.pipeline_layout,
            &PushConstants {
                vertex: MeshEntityVelocityVertexShader::create(
                    camera_buffer.range,
                    draw_data.range,
                    entity_buffer.range,
                    vertex_position_buffer.range,
                ),
                fragment: TransparentEntityIdFragmentShader,
            },
        );

        context.draw_indirect_gpu_scene(&indirect, &draw_count, self.bucket);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("TransparentEntityIdPass destroyed");

        Ok(())
    }
}
