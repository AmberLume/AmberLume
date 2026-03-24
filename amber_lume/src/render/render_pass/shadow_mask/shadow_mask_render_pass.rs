use crate::render::render_pass::render_pass::RenderPass;
use crate::render::render_pass::render_pass_context::RenderPassContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, AttachmentLoadOp, AttachmentStoreOp, BlendFactor, BlendOp, ClearColorValue, ClearValue, ColorComponentFlags, CompareOp, CullModeFlags, Extent2D, Format, FrontFace, ImageLayout, Offset2D, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PolygonMode, PrimitiveTopology, Rect2D, RenderingAttachmentInfoKHR, RenderingInfo, SampleCountFlags, ShaderStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::render_pass::frame_data_context::FrameDataContext;
use crate::render::render_pass::shadow_mask::shadow_mask_push_constants::ShadowMaskPushConstants;
use crate::render::resources::resource_context::ResourceContext;
use crate::resources::dynamic::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::dynamic::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::persistent::persistent_resources::PersistentResources;

pub struct ShadowMaskRenderPass {
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    persistent_resources: Arc<PersistentResources>,

    _pipeline_handle: Arc<ResRef>,

    buffer_manager: Arc<BufferManager>,
}

impl ShadowMaskRenderPass {
    pub fn create(
        resource_context: &ResourceContext,
        pipeline_provider: &ResourceProvider<PipelineBackend>,
        persistent_resources: Arc<PersistentResources>,
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

        let pipeline_handle = pipeline_provider.acquire_sync(pipeline_config);
        let Some(pipeline) = pipeline_provider.get_resource(pipeline_handle.id) else {
            bail!("Failed to acquire Pipeline");
        };

        Ok(Self {
            pipeline,
            pipeline_layout: persistent_resources.pipeline_layouts.global,

            persistent_resources,

            _pipeline_handle: pipeline_handle,

            buffer_manager: resource_context.buffer_manager.clone(),
        })
    }
}

impl RenderPass for ShadowMaskRenderPass {
    type RenderPassData = ();

    fn is_enabled(&self) -> bool {
        true
    }

    fn prepare_data(&self, _context: &FrameDataContext) -> Result<Self::RenderPassData> {
        Ok(())
    }

    fn record_commands(&self, context: &RenderPassContext, _data: Self::RenderPassData) -> Result<()> {
        let depth = &context.render_context.transient_resources.depth;
        context.transition_image_layout(
            depth.image,
            depth.image_subresource_range,
            ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            AccessFlags::SHADER_READ,
            PipelineStageFlags::LATE_FRAGMENT_TESTS,
            PipelineStageFlags::FRAGMENT_SHADER,
        );

        let shadow = &self.persistent_resources.shadows.global_shadow_array;
        context.transition_image_layout(
            shadow.image,
            shadow.image_subresource_range,
            ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            AccessFlags::SHADER_READ,
            PipelineStageFlags::LATE_FRAGMENT_TESTS,
            PipelineStageFlags::FRAGMENT_SHADER,
        );

        let shadow_mask = &context.render_context.transient_resources.shadow_mask;
        context.transition_image_layout(
            shadow_mask.image,
            shadow_mask.image_subresource_range,
            ImageLayout::UNDEFINED,
            ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            AccessFlags::empty(),
            AccessFlags::COLOR_ATTACHMENT_WRITE,
            PipelineStageFlags::TOP_OF_PIPE,
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        );

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
                extent: Extent2D {
                    width: shadow_mask.image_description.extent.width,
                    height: shadow_mask.image_description.extent.height,
                },
            })
            .layer_count(1)
            .color_attachments(color_attachments);

        context.begin_rendering(&rendering_info);

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.set_image_scissor(&context.render_context.transient_resources.shadow_mask);
        context.set_viewport(&context.render_context.transient_resources.shadow_mask);

        context.push_constants(
            self.pipeline_layout,
            &ShadowMaskPushConstants::create(
                self.buffer_manager.scene_buffer.frame(context.frame_index).get().device_address(),
                context.renderer_limits.shadow_map_limits.bias,
                context.renderer_limits.shadow_map_limits.pcf_count,
                context.render_context.transient_resources.depth_descriptor_id,
                self.persistent_resources.shadows.global_shadow_array_descriptor_id,
            ),
        );

        context.draw(3);

        context.end_rendering();

        Ok(())
    }

    fn destroy(&self) -> Result<()> {
        info!("ShadowMaskRenderPass destroyed");

        Ok(())
    }
}
