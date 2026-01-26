use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::render_pass::render_pass::RenderPass;
use crate::render::vulkan::render_pass::render_pass_context::RenderPassContext;
use crate::render::vulkan::render_pass::utils::transition_image_layout;
use crate::render::vulkan::swapchain::swapchain_context::SwapchainContext;
use crate::resources::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};
use crate::resources::pipeline_layout::pipeline_layout_config::{
    PipelineLayoutConfig, PushConstantRange,
};
use crate::resources::resource_hub::ResourceHub;
use anyhow::Result;
use ash::vk::{AccessFlags, AttachmentLoadOp, AttachmentStoreOp, BlendFactor, BlendOp, ColorComponentFlags, CompareOp, CullModeFlags, DescriptorSet, FrontFace, ImageLayout, Offset2D, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PolygonMode, PrimitiveTopology, Rect2D, RenderingAttachmentInfoKHR, RenderingInfoKHR, SampleCountFlags, ShaderStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::render::vulkan::render_pass::ui_render_pass::ui_push_constants::UiPushConstants;
use crate::render::vulkan::resource_context::ResourceContext;
use crate::resources::descriptor_set::descriptor_set_config::DescriptorSetConfig;
use crate::resources::descriptor_set_layout::descriptor_set_layout_config::DescriptorSetLayoutConfig;

pub struct UiRenderPass {
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,
    descriptor_set: DescriptorSet,

    buffer_manager: Arc<BufferManager>,
}

impl UiRenderPass {
    pub fn create(
        resource_context: &ResourceContext,
        swapchain_context: &SwapchainContext,
        resource_hub: Arc<ResourceHub>,
    ) -> Result<Self> {
        let color_format = swapchain_context.format;

        let pipeline_stages = vec![
            PipelineStageConfig {
                shader_name: String::from("shaders/yakui.frag.spv"),
                fn_name: String::from("main"),
                stage: ShaderStageFlags::FRAGMENT,
            },
            PipelineStageConfig {
                shader_name: String::from("shaders/yakui.vert.spv"),
                fn_name: String::from("main"),
                stage: ShaderStageFlags::VERTEX,
            },
        ];

        let descriptor_set_layout_config = DescriptorSetLayoutConfig::default();

        let descriptor_set_config = DescriptorSetConfig {
            descriptor_set_layout_config: descriptor_set_layout_config.clone(),
        };

        let descriptor_set = *resource_hub
            .get_descriptor_set_provider()
            .get_now(&descriptor_set_config);

        let pipeline_layout_config = PipelineLayoutConfig {
            descriptor_set_layout_configs: vec![descriptor_set_layout_config],
            push_constant_ranges: vec![PushConstantRange {
                stage: ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT,
                offset: 0,
                size: size_of::<UiPushConstants>() as u32,
            }],
        };

        let pipeline_layout = *resource_hub
            .get_pipeline_layout_provider()
            .get_now(&pipeline_layout_config);

        let pipeline_config = PipelineConfig {
            stages: pipeline_stages,

            color_formats: vec![color_format],
            depth_format: None,

            cull_mode: CullModeFlags::NONE,
            polygon_mode: PolygonMode::FILL,
            front_face: FrontFace::COUNTER_CLOCKWISE,
            primitive_topology: PrimitiveTopology::TRIANGLE_LIST,

            depth_test: false,
            depth_write: false,
            depth_compare_op: CompareOp::LESS_OR_EQUAL,

            msaa_samples: SampleCountFlags::TYPE_1,

            blend_enabled: true,
            color_blend: Some(BlendConfig {
                blend_op: BlendOp::ADD,
                src_blend: BlendFactor::ONE,
                dst_blend: BlendFactor::ONE_MINUS_SRC_ALPHA,
            }),
            alpha_blend: Some(BlendConfig {
                blend_op: BlendOp::ADD,
                src_blend: BlendFactor::ONE,
                dst_blend: BlendFactor::ONE_MINUS_SRC_ALPHA,
            }),
            color_write_mask: ColorComponentFlags::RGBA,

            pipeline_layout_config,
        };

        let pipeline = *resource_hub
            .get_pipeline_provider()
            .get_now(&pipeline_config);

        Ok(Self {
            pipeline,
            pipeline_layout,
            descriptor_set,
            
            buffer_manager: resource_context.buffer_manager.clone(),
        })
    }
}

impl RenderPass for UiRenderPass {
    fn is_enabled(&self) -> bool {
        true
    }

    fn begin_record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()> {
        transition_image_layout(
            &render_pass_context,
            render_pass_context.swapchain_image,
            ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            AccessFlags::COLOR_ATTACHMENT_WRITE,
            AccessFlags::COLOR_ATTACHMENT_WRITE,
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        );

        let color_attachment = RenderingAttachmentInfoKHR::default()
            .image_view(render_pass_context.swapchain_image.image_view)
            .image_layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(AttachmentLoadOp::LOAD)
            .store_op(AttachmentStoreOp::STORE);

        let color_attachments = vec![color_attachment];

        let rendering_info = RenderingInfoKHR::default()
            .render_area(Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent: render_pass_context.swapchain_image.extent,
            })
            .layer_count(1)
            .color_attachments(&color_attachments);

        render_pass_context.begin_rendering(&rendering_info);

        Ok(())
    }

    fn record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()> {
        render_pass_context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        render_pass_context.set_scissor();
        render_pass_context.set_viewport();

        render_pass_context.bind_index_buffer(&self.buffer_manager.ui_index_buffer);

        render_pass_context.bind_descriptor_sets(self.pipeline_layout, &[self.descriptor_set]);

        render_pass_context.ui_snapshot.draw_calls.iter().for_each(|call| {
            render_pass_context.push_constants(
                self.pipeline_layout,
                ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT,
                &UiPushConstants::create(
                    self.buffer_manager.scene_buffer.device_address.unwrap(),
                    call.texture_index,
                    call.render_mode as u32,
                ),
            );

            render_pass_context.draw_indexed(
                call.index_count,
                call.index_offset,
                call.vertex_offset,
            );
        });

        Ok(())
    }

    fn end_record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()> {
        render_pass_context.end_rendering();

        Ok(())
    }

    fn destroy(&self) -> Result<()> {
        info!("MainRenderPass destroyed");

        Ok(())
    }
}
