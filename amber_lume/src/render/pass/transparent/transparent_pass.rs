use gpu::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_context::PassContext;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::transparent::transparent_push_constants::TransparentPushConstants;
use crate::render::render_graph::pass::Pass;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::pass::draw_bucket::DrawBucket;
use crate::render::pass::draw_pool::DrawPool;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::render::render_graph::virtual_image::render_targets::{ColorTarget, DepthTarget, RenderTargets};
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use gpu::PipelineLayoutType;
use crate::resources::resource_manifest::shaders;
use crate::resources::store::providers::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};
use crate::resources::store::providers::res_ref::ResRef;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, CompareOp, Format, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::sync::Arc;
use tracing::info;

pub struct TransparentPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    target_image: VirtualImage,
    depth: VirtualImage,
    sh_image: VirtualImage,

    brdf_lut_descriptor_id: u32,

    scene_buffer: VirtualBuffer,
    entity_buffer: VirtualBuffer,
    pool: DrawPool,
    bucket: DrawBucket,
    bone_transform: VirtualBuffer,
}

impl TransparentPass {
    pub fn create(
        resources: &PassResources,
        color_format: Format,
        target_image: VirtualImage,
        depth: VirtualImage,
        sh_image: VirtualImage,
        brdf_lut_descriptor_id: u32,
        scene_buffer: VirtualBuffer,
        entity_buffer: VirtualBuffer,
        pool: DrawPool,
        bucket: DrawBucket,
        bone_transform: VirtualBuffer,
    ) -> Result<Self> {
        let pipeline_config = PipelineConfig {
            label: "transparent".to_string(),
            stages: vec![
                PipelineStageConfig::fragment(shaders::TRANSPARENT_FRAG),
                PipelineStageConfig::vertex(shaders::TRANSPARENT_VERT),
            ],
            color_formats: vec![color_format],
            depth_format: Some(resources.render_context.depth_format),
            depth_write: false,
            depth_compare_op: CompareOp::GREATER,
            blend_enabled: true,
            color_blend: Some(BlendConfig::premultiplied_alpha()),
            alpha_blend: Some(BlendConfig::replace()),
            ..PipelineConfig::geometry()
        };

        let _handle = resources.pipeline_provider.acquire_sync(pipeline_config);
        let Some(pipeline) = resources.pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire Pipeline for transparent");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            target_image,
            depth,
            sh_image,

            brdf_lut_descriptor_id,

            scene_buffer,
            entity_buffer,
            pool,
            bucket,
            bone_transform,
        })
    }
}

impl Pass for TransparentPass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("transparent")
    }

    fn is_enabled(&self, _context: &FrameDataContext) -> bool {
        true
    }

    fn prepare_data(
        &self,
        _context: &FrameDataContext,
        _buffer_scope: &mut BufferResourceScope,
        _allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        Ok(())
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .write_image(
                self.target_image,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_WRITE | AccessFlags::COLOR_ATTACHMENT_READ,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            )
            .read_image(
                self.depth,
                ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
                PipelineStageFlags::EARLY_FRAGMENT_TESTS | PipelineStageFlags::LATE_FRAGMENT_TESTS,
            )
            .read_image(
                self.sh_image,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_buffer(
                self.scene_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER | PipelineStageFlags::FRAGMENT_SHADER,
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
                PipelineStageFlags::VERTEX_SHADER | PipelineStageFlags::FRAGMENT_SHADER,
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
                image: self.target_image,
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

    fn record_commands(&self, context: &PassContext, image_scope: &ImageResourceScope, buffer_scope: &BufferResourceScope, _data: Self::PassData) -> Result<()> {
        let sh_image = image_scope.get_physical_image(self.sh_image);
        let sh_descriptor_id = sh_image
            .descriptors
            .full
            .expect("Transparent sh image must have a sampled descriptor");

        let scene_buffer = buffer_scope.get_physical_buffer(self.scene_buffer);
        let entity_buffer = buffer_scope.get_physical_buffer(self.entity_buffer);
        let draw_count = buffer_scope.get_physical_buffer(self.pool.draw_count);
        let indirect = buffer_scope.get_physical_buffer(self.pool.indirect);
        let draw_data = buffer_scope.get_physical_buffer(self.pool.draw_data);
        let bone_transform_buffer = buffer_scope.get_physical_buffer(self.bone_transform);

        context.bind_index_buffer();

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &TransparentPushConstants::create(
                &scene_buffer,
                &draw_data,
                context.resource_buffers.vertex_buffer,
                &entity_buffer,
                context.resource_buffers.submesh_buffer,
                context.resource_buffers.material_buffer,
                &bone_transform_buffer,
                sh_descriptor_id,
                self.brdf_lut_descriptor_id,
            ),
        );

        context.draw_indirect_gpu_scene(&indirect, &draw_count, self.bucket);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("TransparentPass destroyed");

        Ok(())
    }
}
