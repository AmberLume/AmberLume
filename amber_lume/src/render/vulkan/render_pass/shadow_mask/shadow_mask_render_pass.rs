use crate::render::vulkan::render_pass::render_pass::RenderPass;
use crate::render::vulkan::render_pass::render_pass_context::RenderPassContext;
use crate::render::vulkan::render_pass::utils::transition_image_layout;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, AttachmentLoadOp, AttachmentStoreOp, BlendFactor, BlendOp, ClearColorValue, ClearValue, ColorComponentFlags, CompareOp, CullModeFlags, Extent2D, Format, FrontFace, ImageLayout, Offset2D, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PolygonMode, PrimitiveTopology, Rect2D, RenderingAttachmentInfoKHR, RenderingInfoKHR, SampleCountFlags, ShaderStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::render_pass::shadow_mask::shadow_mask_push_constants::ShadowMaskPushConstants;
use crate::render::vulkan::resource_context::ResourceContext;
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
    fn is_enabled(&self) -> bool {
        true
    }

    fn begin_record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()> {
        let depth = &render_pass_context.render_context.transient_resources.depth;
        transition_image_layout(
            &render_pass_context,
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
        transition_image_layout(
            &render_pass_context,
            shadow.image,
            shadow.image_subresource_range,
            ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            AccessFlags::SHADER_READ,
            PipelineStageFlags::LATE_FRAGMENT_TESTS,
            PipelineStageFlags::FRAGMENT_SHADER,
        );

        let shadow_mask = &render_pass_context.render_context.transient_resources.shadow_mask;
        transition_image_layout(
            &render_pass_context,
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
        let rendering_info = RenderingInfoKHR::default()
            .render_area(Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent: Extent2D {
                    width: shadow_mask.image_description.extent.width,
                    height: shadow_mask.image_description.extent.height,
                },
            })
            .layer_count(1)
            .color_attachments(color_attachments);

        render_pass_context.begin_rendering(&rendering_info);

        Ok(())
    }

    fn record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()> {
        render_pass_context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        render_pass_context.set_scissor(&render_pass_context.render_context.transient_resources.shadow_mask);
        render_pass_context.set_viewport(&render_pass_context.render_context.transient_resources.shadow_mask);

        render_pass_context.push_constants(
            self.pipeline_layout,
            &ShadowMaskPushConstants::create(
                self.buffer_manager.scene_buffer.handle.device_address.unwrap(),
                render_pass_context.renderer_limits.shadow_map_limits.bias,
                render_pass_context.renderer_limits.shadow_map_limits.pcf_count,
                render_pass_context.render_context.transient_resources.depth_descriptor_id,
                self.persistent_resources.shadows.global_shadow_array_descriptor_id,
            ),
        );

        unsafe {
            render_pass_context.device_context.device.cmd_draw(
                render_pass_context.command_recording.command_buffer,
                3, 1, 0, 0
            );
        }

        Ok(())
    }

    fn end_record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()> {
        render_pass_context.end_rendering();

        Ok(())
    }

    fn destroy(&self) -> Result<()> {
        info!("ShadowMaskRenderPass destroyed");

        Ok(())
    }
}
