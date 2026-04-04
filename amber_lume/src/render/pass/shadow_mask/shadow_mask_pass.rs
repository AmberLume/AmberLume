use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, AttachmentLoadOp, AttachmentStoreOp, BlendFactor, BlendOp, ClearColorValue, ClearValue, ColorComponentFlags, CompareOp, CullModeFlags, Format, FrontFace, ImageLayout, Offset2D, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PolygonMode, PrimitiveTopology, Rect2D, RenderingAttachmentInfoKHR, RenderingInfo, SampleCountFlags, ShaderStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::ids::FrameIndex;
use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::shadow_mask::shadow_mask_push_constants::ShadowMaskPushConstants;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::render_graph::resource_registry::resource_registry::ResourceRegistry;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::render::resources::resource_context::ResourceContext;
use crate::resources::dynamic::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::dynamic::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::persistent::persistent_resources::PersistentResources;
use crate::resources::pipeline_layout_registry::{PipelineLayoutRegistry, PipelineLayoutType};

pub struct ShadowMaskPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    buffer_manager: Arc<BufferManager>,

    persistent_resources: Arc<PersistentResources>,

    depth: VirtualImage,
    shadows: VirtualImage,
    shadow_mask: VirtualImage,
}

impl ShadowMaskPass {
    pub fn create(
        resource_context: &ResourceContext,
        pipeline_provider: &ResourceProvider<PipelineBackend>,
        pipeline_layout_registry: &PipelineLayoutRegistry,
        persistent_resources: Arc<PersistentResources>,
        depth: VirtualImage,
        shadows: VirtualImage,
        shadow_mask: VirtualImage,
    ) -> Result<Self> {
        let pipeline_stages = vec![
            PipelineStageConfig {
                shader_name: String::from("shaders/shadow_mask/shadow_mask.frag.spv"),
                fn_name: String::from("main"),
                stage: ShaderStageFlags::FRAGMENT,
            },
            PipelineStageConfig {
                shader_name: String::from("shaders/shadow_mask/shadow_mask.vert.spv"),
                fn_name: String::from("main"),
                stage: ShaderStageFlags::VERTEX,
            },
        ];

        let pipeline_config = PipelineConfig {
            label: "shadow_mask".to_string(),

            stages: pipeline_stages,

            color_formats: vec![Format::R8_UNORM],
            depth_format: None,

            cull_mode: CullModeFlags::NONE,
            polygon_mode: PolygonMode::FILL,
            front_face: FrontFace::COUNTER_CLOCKWISE,
            primitive_topology: PrimitiveTopology::TRIANGLE_LIST,

            depth_bias_enable: false,
            depth_bias_constant_factor: 0.0,
            depth_bias_slope_factor: 0.0,

            depth_test: false,
            depth_write: false,
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

            buffer_manager: resource_context.buffer_manager.clone(),

            persistent_resources,

            depth,
            shadows,
            shadow_mask,
        })
    }
}

impl Pass for ShadowMaskPass {
    type PassData = ();
    type Statistics = ();

    fn name(&self) -> String {
        String::from("shadow_mask")
    }
    
    fn is_enabled(&self) -> bool {
        true
    }

    fn prepare_data(&self, _context: &FrameDataContext) -> Result<Self::PassData> {
        Ok(())
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .image(
                self.depth,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            )
            .image(
                self.shadows,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            )
            .image(
                self.shadow_mask,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_WRITE,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            );
    }

    fn record_commands(&self, context: &PassContext, resource_registry: &ResourceRegistry, _data: Self::PassData) -> Result<()> {
        let shadow_mask = resource_registry.get(self.shadow_mask);

        let color_attachment = RenderingAttachmentInfoKHR::default()
            .image_view(shadow_mask.image_view)
            .image_layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(AttachmentLoadOp::CLEAR)
            .store_op(AttachmentStoreOp::STORE)
            .clear_value(ClearValue {
                color: ClearColorValue {
                    float32: [1.0; 4]
                },
            });

        let color_attachments = &[color_attachment];
        let rendering_info = RenderingInfo::default()
            .render_area(Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent: shadow_mask.extent,
            })
            .layer_count(1)
            .color_attachments(color_attachments);

        context.begin_rendering(&rendering_info);

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.set_image_scissor(&shadow_mask);
        context.set_viewport(&shadow_mask);

        context.push_constants(
            self.pipeline_layout,
            &ShadowMaskPushConstants::create(
                self.buffer_manager.scene_buffer.frame(context.frame_index),
                context.renderer_limits.shadow_map_limits.bias,
                context.renderer_limits.shadow_map_limits.pcf_count,
                context.render_context.transient_resources.depth.id,
                self.persistent_resources.shadows.global_shadow_array_descriptor_id,
            ),
        );

        context.draw(3);

        context.end_rendering();

        Ok(())
    }

    fn statistics(&self, _frame_index: FrameIndex) -> Self::Statistics {
        ()
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("ShadowMaskRenderPass destroyed");

        Ok(())
    }
}
