use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::render_pass::depth::depth_push_constants::DepthPushConstants;
use crate::render::vulkan::render_pass::render_pass::RenderPass;
use crate::render::vulkan::render_pass::render_pass_context::RenderPassContext;
use crate::render::vulkan::render_pass::utils::transition_image_layout;
use crate::render::vulkan::renderer::render_context::RenderContext;
use crate::resources::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};
use crate::resources::pipeline_layout::pipeline_layout_config::{
    PipelineLayoutConfig, PushConstantRange,
};
use crate::resources::resource_hub::ResourceHub;
use anyhow::Result;
use ash::vk::{AccessFlags, AttachmentLoadOp, AttachmentStoreOp, BlendFactor, BlendOp, ClearDepthStencilValue, ClearValue, ColorComponentFlags, CompareOp, CullModeFlags, FrontFace, ImageLayout, Offset2D, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PolygonMode, PrimitiveTopology, Rect2D, RenderingAttachmentInfoKHR, RenderingInfoKHR, SampleCountFlags, ShaderStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::render::vulkan::resource_context::ResourceContext;

pub struct DepthRenderPass {
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    buffer_manager: Arc<BufferManager>,
}

impl DepthRenderPass {
    pub fn create(
        resource_context: &ResourceContext,
        render_context: &RenderContext,
        resource_hub: Arc<ResourceHub>,
    ) -> Result<Self> {
        let format = render_context.render_targets.depth_vulkan_image.format;

        let pipeline_stages = vec![
            PipelineStageConfig {
                shader_name: String::from("shaders/depth.frag.spv"),
                fn_name: String::from("main"),
                stage: ShaderStageFlags::FRAGMENT,
            },
            PipelineStageConfig {
                shader_name: String::from("shaders/depth.vert.spv"),
                fn_name: String::from("main"),
                stage: ShaderStageFlags::VERTEX,
            },
        ];

        let pipeline_layout_config = PipelineLayoutConfig {
            descriptor_set_layout_configs: vec![],
            push_constant_ranges: vec![PushConstantRange {
                stage: ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT,
                offset: 0,
                size: size_of::<DepthPushConstants>() as u32,
            }],
        };

        let pipeline_layout = *resource_hub
            .get_pipeline_layout_provider()
            .get_now(&pipeline_layout_config);

        let pipeline_config = PipelineConfig {
            stages: pipeline_stages,

            color_formats: vec![],
            depth_format: Some(format),

            cull_mode: CullModeFlags::BACK,
            polygon_mode: PolygonMode::FILL,
            front_face: FrontFace::COUNTER_CLOCKWISE,
            primitive_topology: PrimitiveTopology::TRIANGLE_LIST,

            depth_test: true,
            depth_write: true,
            depth_compare_op: CompareOp::LESS,

            msaa_samples: SampleCountFlags::TYPE_1,

            blend_enabled: false,
            color_blend: Some(BlendConfig {
                blend_op: BlendOp::ADD,
                src_blend: BlendFactor::ONE,
                dst_blend: BlendFactor::ZERO,
            }),
            alpha_blend: None,
            color_write_mask: ColorComponentFlags::RGBA,

            pipeline_layout_config,
        };

        let pipeline = *resource_hub
            .get_pipeline_provider()
            .get_now(&pipeline_config);

        Ok(Self {
            pipeline,
            pipeline_layout,

            buffer_manager: resource_context.buffer_manager.clone(),
        })
    }
}

impl RenderPass for DepthRenderPass {
    fn is_enabled(&self) -> bool {
        true
    }

    fn begin_record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()> {
        let depth_image = &render_pass_context
            .render_context
            .render_targets
            .depth_vulkan_image;

        transition_image_layout(
            &render_pass_context,
            depth_image,
            ImageLayout::UNDEFINED,
            ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            AccessFlags::empty(),
            AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            PipelineStageFlags::TOP_OF_PIPE,
            PipelineStageFlags::EARLY_FRAGMENT_TESTS | PipelineStageFlags::LATE_FRAGMENT_TESTS,
        );

        let depth_attachment = RenderingAttachmentInfoKHR::default()
            .image_view(depth_image.image_view)
            .image_layout(ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .load_op(AttachmentLoadOp::CLEAR)
            .store_op(AttachmentStoreOp::STORE)
            .clear_value(ClearValue {
                depth_stencil: ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            });

        let rendering_info = RenderingInfoKHR::default()
            .render_area(Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent: depth_image.extent,
            })
            .layer_count(1)
            .depth_attachment(&depth_attachment);

        render_pass_context.begin_rendering(&rendering_info);

        Ok(())
    }

    fn record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()> {
        render_pass_context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        render_pass_context.set_scissor();
        render_pass_context.set_viewport();

        render_pass_context.bind_index_buffer(&self.buffer_manager.index_buffer);

        render_pass_context.push_constants(
            self.pipeline_layout,
            ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT,
            &DepthPushConstants::create(
                self.buffer_manager.scene_buffer.device_address.unwrap(),
            ),
        );

        render_pass_context.draw_indirect_gpu_scene(
            &self.buffer_manager.indirect_buffer,
            &self.buffer_manager.draw_count_buffer,
        );

        Ok(())
    }

    fn end_record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()> {
        render_pass_context.end_rendering();

        Ok(())
    }

    fn destroy(&self) -> Result<()> {
        info!("DepthRenderPass destroyed");

        Ok(())
    }
}
