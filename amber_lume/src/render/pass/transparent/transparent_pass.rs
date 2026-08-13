use render_graph::ReadbackScope;
use gpu::ResourceFactories;
use render_graph::FrameContext;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::transparent::transparent_push_constants::TransparentPushConstants;
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
use pipeline_store::BlendConfig;
use pipeline_store::PipelineConfig;
use pipeline_store::PipelineStageConfig;
use resource_residency::ResRef;
use anyhow::{bail, Result};
use ash::vk::{Buffer, AccessFlags, CompareOp, DeviceAddress, Format, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
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

    vertex_buffer: DeviceAddress,
    submesh_buffer: DeviceAddress,
    material_buffer: DeviceAddress,
    index_buffer_handle: Buffer,
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

            vertex_buffer: resources.resource_buffers.vertex_buffer,
            submesh_buffer: resources.resource_buffers.submesh_buffer,
            material_buffer: resources.resource_buffers.material_buffer,
            index_buffer_handle: resources.resource_buffers.index_buffer_handle,
        })
    }
}

impl Pass for TransparentPass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("transparent")
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

    fn record_commands(
        &self,
        context: &FrameContext, 
        image_scope: &ImageResourceScope,
        buffer_scope: &BufferResourceScope, 
        
        _readback_scope: &ReadbackScope,
        
        _data: Self::PassData,
    ) -> Result<()> {
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

        context.bind_index_buffer(self.index_buffer_handle, 0);

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &TransparentPushConstants::create(
                &scene_buffer,
                &draw_data,
                self.vertex_buffer,
                &entity_buffer,
                self.submesh_buffer,
                self.material_buffer,
                &bone_transform_buffer,
                sh_descriptor_id.inner,
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
