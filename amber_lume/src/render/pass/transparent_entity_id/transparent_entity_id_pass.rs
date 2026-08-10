use gpu::ResourceFactories;
use render_graph::FrameContext;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::transparent_entity_id::transparent_entity_id_push_constants::TransparentEntityIdPushConstants;
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
use render_graph::HeapAllocator;
use render_graph::DrawBucket;
use crate::render::pass::draw_pool::DrawPool;
use render_graph::VirtualBuffer;
use render_graph::{ColorTarget, DepthTarget, RenderTargets};
use render_graph::VirtualImage;
use render_graph::BufferResourceScope;
use render_graph::DataResourceScope;
use render_graph::ImageResourceScope;
use gpu::PipelineLayoutType;
use crate::resource_manifest::shaders;
use pipeline_store::PipelineConfig;
use pipeline_store::PipelineStageConfig;
use resource_residency::ResRef;
use anyhow::{bail, Result};
use ash::vk::{Buffer, AccessFlags, CompareOp, DeviceAddress, Format, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::sync::Arc;
use tracing::info;

pub struct TransparentEntityIdPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    entity_id_image: VirtualImage,
    depth: VirtualImage,

    scene_buffer: VirtualBuffer,
    entity_buffer: VirtualBuffer,
    pool: DrawPool,
    bucket: DrawBucket,
    bone_transform: VirtualBuffer,

    vertex_buffer: DeviceAddress,
    index_buffer_handle: Buffer,
}

impl TransparentEntityIdPass {
    pub fn create(
        resources: &PassResources,
        entity_id_image: VirtualImage,
        depth: VirtualImage,
        scene_buffer: VirtualBuffer,
        entity_buffer: VirtualBuffer,
        pool: DrawPool,
        bucket: DrawBucket,
        bone_transform: VirtualBuffer,
    ) -> Result<Self> {
        let pipeline_config = PipelineConfig {
            label: "transparent_entity_id".to_string(),
            stages: vec![
                PipelineStageConfig::fragment(shaders::TRANSPARENT_ENTITY_ID_FRAG),
                PipelineStageConfig::vertex(shaders::TRANSPARENT_ENTITY_ID_VERT),
            ],
            color_formats: vec![Format::R32_UINT],
            depth_format: Some(resources.render_context.depth_format),
            depth_write: false,
            depth_compare_op: CompareOp::GREATER,
            ..PipelineConfig::geometry()
        };

        let _handle = resources.pipeline_provider.acquire_sync(pipeline_config);
        let Some(pipeline) = resources.pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire Pipeline for transparent_entity_id");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            entity_id_image,
            depth,

            scene_buffer,
            entity_buffer,
            pool,
            bucket,
            bone_transform,

            vertex_buffer: resources.resource_buffers.vertex_buffer,
            index_buffer_handle: resources.resource_buffers.index_buffer_handle,
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
        _data_scope: &mut DataResourceScope,
        _buffer_scope: &mut BufferResourceScope,
        _allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        Ok(())
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
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
                self.scene_buffer,
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
                self.bone_transform,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER,
            );
    }

    fn render_targets(&self) -> Option<RenderTargets> {
        Some(RenderTargets {
            color: vec![ColorTarget {
                image: self.entity_id_image,
                mip: None,
                clear: None,
            }],
            depth: Some(DepthTarget {
                image: self.depth,
                clear: None,
            }),
            view_mask: 0,
        })
    }

    fn record_commands(&self, context: &FrameContext, _image_scope: &ImageResourceScope, buffer_scope: &BufferResourceScope, _data: Self::PassData) -> Result<()> {
        let scene_buffer = buffer_scope.get_physical_buffer(self.scene_buffer);
        let entity_buffer = buffer_scope.get_physical_buffer(self.entity_buffer);
        let draw_count = buffer_scope.get_physical_buffer(self.pool.draw_count);
        let indirect = buffer_scope.get_physical_buffer(self.pool.indirect);
        let draw_data = buffer_scope.get_physical_buffer(self.pool.draw_data);
        let bone_transform_buffer = buffer_scope.get_physical_buffer(self.bone_transform);

        context.bind_index_buffer(self.index_buffer_handle, 0);

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &TransparentEntityIdPushConstants::create(
                &scene_buffer,
                &draw_data,
                self.vertex_buffer,
                &entity_buffer,
                &bone_transform_buffer,
            ),
        );

        context.draw_indirect_gpu_scene(&indirect, &draw_count, self.bucket);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("TransparentEntityIdPass destroyed");

        Ok(())
    }
}
