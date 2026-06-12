use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, BlendFactor, BlendOp, ColorComponentFlags, CompareOp, CullModeFlags, Format, FrontFace, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PolygonMode, PrimitiveTopology, SampleCountFlags, ShaderStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::shadows::shadows_push_constants::ShadowsPushConstants;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::render::render_graph::virtual_image::render_targets::{DepthTarget, RenderTargets};
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::resources::binding_layout::pipeline_layout_registry::{PipelineLayoutRegistry, PipelineLayoutType};
use crate::resources::store::providers::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::store::providers::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::store::providers::resource_provider::ResourceProvider;
use crate::resources::resource_manifest::shaders;

pub struct ShadowsPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    shadows_image: VirtualImage,
    view_mask: u32,

    entity_buffer: VirtualBuffer,
    shadow_cascades_buffer: VirtualBuffer,
    draw_count_shadow: VirtualBuffer,
    indirect_shadow: VirtualBuffer,
    draw_data_shadow: VirtualBuffer,
    bone_transform: VirtualBuffer,
}

impl ShadowsPass {
    pub fn create(
        pipeline_provider: &ResourceProvider<PipelineBackend>,
        pipeline_layout_registry: &PipelineLayoutRegistry,
        cascade_count: u32,
        depth_format: Format,
        shadows_image: VirtualImage,
        entity_buffer: VirtualBuffer,
        shadow_cascades_buffer: VirtualBuffer,
        draw_count_shadow: VirtualBuffer,
        indirect_shadow: VirtualBuffer,
        draw_data_shadow: VirtualBuffer,
        bone_transform: VirtualBuffer,
    ) -> Result<Self> {
        let view_mask = (1u32 << cascade_count) - 1;

        let pipeline_stages = vec![
            PipelineStageConfig {
                shader_name: shaders::SHADOWS_VERT,
                fn_name: String::from("main"),
                stage: ShaderStageFlags::VERTEX,
            },
        ];

        let pipeline_config = PipelineConfig {
            label: "shadows".to_string(),

            stages: pipeline_stages,

            color_formats: vec![],
            depth_format: Some(depth_format),
            view_mask,

            cull_mode: CullModeFlags::BACK,
            polygon_mode: PolygonMode::FILL,
            front_face: FrontFace::COUNTER_CLOCKWISE,
            primitive_topology: PrimitiveTopology::TRIANGLE_LIST,

            depth_bias_enable: true,
            depth_bias_constant_factor: 1.5,
            depth_bias_slope_factor: 2.0,

            depth_test: true,
            depth_write: true,
            depth_compare_op: CompareOp::LESS_OR_EQUAL,

            msaa_samples: SampleCountFlags::TYPE_1,

            blend_enabled: false,
            color_blend: Some(BlendConfig {
                blend_op: BlendOp::ADD,
                src_blend: BlendFactor::ONE,
                dst_blend: BlendFactor::ZERO,
            }),
            alpha_blend: None,
            color_write_mask: ColorComponentFlags::RGBA,
        };

        let _handle = pipeline_provider.acquire_sync(pipeline_config);
        let Some(pipeline) = pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire Pipeline");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: pipeline_layout_registry.get(PipelineLayoutType::General),

            shadows_image,
            view_mask,

            entity_buffer,
            shadow_cascades_buffer,
            draw_count_shadow,
            indirect_shadow,
            draw_data_shadow,
            bone_transform,
        })
    }
}

impl Pass for ShadowsPass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("shadows")
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
                self.draw_count_shadow,
                AccessFlags::INDIRECT_COMMAND_READ,
                PipelineStageFlags::DRAW_INDIRECT,
            )
            .read_buffer(
                self.indirect_shadow,
                AccessFlags::INDIRECT_COMMAND_READ,
                PipelineStageFlags::DRAW_INDIRECT,
            )
            .read_buffer(
                self.draw_data_shadow,
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
            color: Vec::new(),
            depth: Some(DepthTarget { image: self.shadows_image, clear: Some(1.0) }),
            view_mask: self.view_mask,
        })
    }

    fn record_commands(&self, context: &PassContext, _image_scope: &ImageResourceScope, buffer_scope: &BufferResourceScope, _data: Self::PassData) -> Result<()> {
        let entity_buffer = buffer_scope.get_physical_buffer(self.entity_buffer);
        let shadow_cascades_buffer = buffer_scope.get_physical_buffer(self.shadow_cascades_buffer);
        let draw_count_shadow = buffer_scope.get_physical_buffer(self.draw_count_shadow);
        let indirect_shadow = buffer_scope.get_physical_buffer(self.indirect_shadow);
        let draw_data_shadow = buffer_scope.get_physical_buffer(self.draw_data_shadow);
        let bone_transform_buffer = buffer_scope.get_physical_buffer(self.bone_transform);

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.bind_index_buffer();

        context.push_constants(
            self.pipeline_layout,
            &ShadowsPushConstants::create(
                &draw_data_shadow,
                &entity_buffer,
                context.resource_buffers.vertex_buffer,
                &bone_transform_buffer,
                &shadow_cascades_buffer,
            ),
        );
        context.draw_indirect_gpu_scene(
            &indirect_shadow,
            &draw_count_shadow,
        );

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("ShadowsRenderPass destroyed");

        Ok(())
    }
}
