use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use crate::render::pass::pass_resources::PassResources;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, CompareOp, CullModeFlags, Format, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::shadows::translucent_shadows::translucent_shadows_push_constants::TranslucentShadowsPushConstants;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::render::render_graph::virtual_image::render_targets::{ClearColor, ColorTarget, DepthTarget, RenderTargets};
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::resources::binding_layout::pipeline_layout_registry::PipelineLayoutType;
use crate::resources::store::providers::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::resource_manifest::shaders;

pub struct TranslucentShadowsPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    shadows_image: VirtualImage,
    transmittance_image: VirtualImage,
    view_mask: u32,

    entity_buffer: VirtualBuffer,
    shadow_cascades_buffer: VirtualBuffer,
    draw_count_shadow_blend: VirtualBuffer,
    indirect_shadow_blend: VirtualBuffer,
    draw_data_shadow_blend: VirtualBuffer,
    bone_transform: VirtualBuffer,
}

impl TranslucentShadowsPass {
    pub const TRANSMITTANCE_FORMAT: Format = Format::R8_UNORM;

    pub fn create(
        resources: &PassResources,
        cascade_count: u32,
        depth_format: Format,
        shadows_image: VirtualImage,
        transmittance_image: VirtualImage,
        entity_buffer: VirtualBuffer,
        shadow_cascades_buffer: VirtualBuffer,
        draw_count_shadow_blend: VirtualBuffer,
        indirect_shadow_blend: VirtualBuffer,
        draw_data_shadow_blend: VirtualBuffer,
        bone_transform: VirtualBuffer,
    ) -> Result<Self> {
        let view_mask = (1u32 << cascade_count) - 1;

        let pipeline_config = PipelineConfig {
            label: "translucent_shadows".to_string(),
            stages: vec![
                PipelineStageConfig::vertex(shaders::TRANSLUCENT_SHADOWS_VERT),
                PipelineStageConfig::fragment(shaders::TRANSLUCENT_SHADOWS_FRAG),
            ],
            color_formats: vec![Self::TRANSMITTANCE_FORMAT],
            depth_format: Some(depth_format),
            view_mask,
            cull_mode: CullModeFlags::NONE,
            depth_write: false,
            depth_compare_op: CompareOp::LESS_OR_EQUAL,
            blend_enabled: true,
            color_blend: Some(BlendConfig::MULTIPLICATIVE),
            alpha_blend: Some(BlendConfig::MULTIPLICATIVE),
            ..PipelineConfig::geometry()
        };

        let _handle = resources.pipeline_provider.acquire_sync(pipeline_config);
        let Some(pipeline) = resources.pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire Pipeline for TranslucentShadows");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            shadows_image,
            transmittance_image,
            view_mask,

            entity_buffer,
            shadow_cascades_buffer,
            draw_count_shadow_blend,
            indirect_shadow_blend,
            draw_data_shadow_blend,
            bone_transform,
        })
    }
}

impl Pass for TranslucentShadowsPass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("translucent_shadows")
    }

    fn is_enabled(&self) -> bool {
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
                self.transmittance_image,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_WRITE | AccessFlags::COLOR_ATTACHMENT_READ,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            )
            .read_image(
                self.shadows_image,
                ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
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
                self.draw_count_shadow_blend,
                AccessFlags::INDIRECT_COMMAND_READ,
                PipelineStageFlags::DRAW_INDIRECT,
            )
            .read_buffer(
                self.indirect_shadow_blend,
                AccessFlags::INDIRECT_COMMAND_READ,
                PipelineStageFlags::DRAW_INDIRECT,
            )
            .read_buffer(
                self.draw_data_shadow_blend,
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
                image: self.transmittance_image,
                mip: None,
                clear: Some(ClearColor::Float([1.0; 4])),
            }],
            depth: Some(DepthTarget { image: self.shadows_image, clear: None }),
            view_mask: self.view_mask,
        })
    }

    fn record_commands(&self, context: &PassContext, _image_scope: &ImageResourceScope, buffer_scope: &BufferResourceScope, _data: Self::PassData) -> Result<()> {
        let entity_buffer = buffer_scope.get_physical_buffer(self.entity_buffer);
        let shadow_cascades_buffer = buffer_scope.get_physical_buffer(self.shadow_cascades_buffer);
        let draw_count_shadow_blend = buffer_scope.get_physical_buffer(self.draw_count_shadow_blend);
        let indirect_shadow_blend = buffer_scope.get_physical_buffer(self.indirect_shadow_blend);
        let draw_data_shadow_blend = buffer_scope.get_physical_buffer(self.draw_data_shadow_blend);
        let bone_transform_buffer = buffer_scope.get_physical_buffer(self.bone_transform);

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.bind_index_buffer();

        context.push_constants(
            self.pipeline_layout,
            &TranslucentShadowsPushConstants::create(
                &draw_data_shadow_blend,
                &entity_buffer,
                context.resource_buffers.vertex_buffer,
                &bone_transform_buffer,
                &shadow_cascades_buffer,
                context.resource_buffers.submesh_buffer,
                context.resource_buffers.material_buffer,
            ),
        );
        context.draw_indirect_gpu_scene(
            &indirect_shadow_blend,
            &draw_count_shadow_blend,
        );

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("TranslucentShadowsPass destroyed");

        Ok(())
    }
}
