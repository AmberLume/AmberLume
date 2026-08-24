use render_graph::ReadbackScope;
use std::sync::Arc;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, CompareOp, Format, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use tracing::info;
use gpu::ResourceFactories;
use crate::render::pass::depth::depth_push_constants::DepthPushConstants;
use render_graph::FrameContext;
use crate::render::pass::pass_resources::PassResources;
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
use render_graph::ImageResourceScope;
use render_graph::BufferResourceScope;
use render_graph::DataResourceScope;
use render_graph::HeapAllocator;
use render_graph::DrawBucket;
use crate::render::pass::draw_pool::DrawPool;
use render_graph::VirtualBuffer;
use render_graph::{ClearColor, ColorTarget, DepthTarget, RenderTargets};
use render_graph::VirtualImage;
use gpu::PipelineLayoutType;
use crate::resource_manifest::shaders;
use pipeline_store::PipelineConfig;
use pipeline_store::PipelineStageConfig;
use resource_residency::ResRef;

pub struct DepthPrepass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    depth: VirtualImage,
    normal: VirtualImage,
    velocity: VirtualImage,

    scene_buffer: VirtualBuffer,
    entity_buffer: VirtualBuffer,
    entity_motion_buffer: VirtualBuffer,
    pool: DrawPool,
    bucket: DrawBucket,
    bone_transform: VirtualBuffer,
    mesh_vertex_buffer: VirtualBuffer,
    mesh_vertex_skin_buffer: VirtualBuffer,
    submesh_buffer: VirtualBuffer,
    index_buffer: VirtualBuffer,
}

impl DepthPrepass {
    pub fn create(
        resources: &PassResources,
        depth: VirtualImage,
        normal: VirtualImage,
        normal_format: Format,
        velocity: VirtualImage,
        velocity_format: Format,
        scene_buffer: VirtualBuffer,
        entity_buffer: VirtualBuffer,
        entity_motion_buffer: VirtualBuffer,
        pool: DrawPool,
        bucket: DrawBucket,
        bone_transform: VirtualBuffer,
    ) -> Result<Self> {
        let pipeline_config = PipelineConfig {
            label: "depth_prepass".to_string(),
            stages: vec![
                PipelineStageConfig::fragment(shaders::DEPTH_FRAG),
                PipelineStageConfig::vertex(shaders::DEPTH_VERT),
            ],
            color_formats: vec![normal_format, velocity_format],
            depth_format: Some(resources.render_context.depth_format),
            depth_compare_op: CompareOp::GREATER,
            ..PipelineConfig::geometry()
        };

        let _handle = resources.pipeline_provider.acquire_sync(pipeline_config)?;
        let Some(pipeline) = resources.pipeline_provider.with_resource(_handle.id, |pipeline| *pipeline) else {
            bail!("Failed to acquire Pipeline for depth_prepass");
        };

        Ok(Self {
            _handle,

            pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            depth,
            normal,
            velocity,

            scene_buffer,
            entity_buffer,
            entity_motion_buffer,
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

impl Pass for DepthPrepass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("depth_prepass")
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
                self.depth,
                ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE | AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
                PipelineStageFlags::EARLY_FRAGMENT_TESTS | PipelineStageFlags::LATE_FRAGMENT_TESTS,
            )
            .write_image(
                self.normal,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_WRITE,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            )
            .write_image(
                self.velocity,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_WRITE,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
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
                self.entity_motion_buffer,
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
            color: vec![
                ColorTarget {
                    image: self.normal,
                    mip: None,
                    clear: None,
                },
                ColorTarget {
                    image: self.velocity,
                    mip: None,
                    clear: Some(ClearColor::Float([0.0, 0.0, 0.0, 0.0])),
                },
            ],
            depth: Some(DepthTarget {
                image: self.depth,
                clear: Some(0.0),
            }),
            view_mask: 0,
        })
    }

    fn record_commands(
        &self,
        context: &FrameContext,
        _image_scope: &ImageResourceScope,
        buffer_scope: &BufferResourceScope,
        _readback_scope: &ReadbackScope,
        _data: Self::PassData,
    ) -> Result<()> {
        let mesh_vertex_buffer = buffer_scope.get_physical_buffer(self.mesh_vertex_buffer);
        let mesh_vertex_skin_buffer = buffer_scope.get_physical_buffer(self.mesh_vertex_skin_buffer);
        let submesh_buffer = buffer_scope.get_physical_buffer(self.submesh_buffer);
        let index_buffer = buffer_scope.get_physical_buffer(self.index_buffer);
        let scene_buffer = buffer_scope.get_physical_buffer(self.scene_buffer);
        let entity_buffer = buffer_scope.get_physical_buffer(self.entity_buffer);
        let entity_motion_buffer = buffer_scope.get_physical_buffer(self.entity_motion_buffer);
        let draw_count = buffer_scope.get_physical_buffer(self.pool.draw_count);
        let indirect = buffer_scope.get_physical_buffer(self.pool.indirect);
        let draw_data = buffer_scope.get_physical_buffer(self.pool.draw_data);
        let bone_transform_buffer = buffer_scope.get_physical_buffer(self.bone_transform);

        context.bind_index_buffer(index_buffer.range);

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);
        context.push_constants(
            self.pipeline_layout,
            &DepthPushConstants::create(
                scene_buffer.range,
                draw_data.range,
                entity_buffer.range,
                entity_motion_buffer.range,
                submesh_buffer.range,
                mesh_vertex_buffer.range,
                mesh_vertex_skin_buffer.range,
                bone_transform_buffer.range,
            ),
        );
        context.draw_indirect_gpu_scene(&indirect, &draw_count, self.bucket);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("DepthPrepass destroyed");

        Ok(())
    }
}
