use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::render_pass::render_pass::RenderPass;
use crate::render::vulkan::render_pass::render_pass_context::RenderPassContext;
use crate::render::vulkan::renderer::render_context::RenderContext;
use crate::render::vulkan::swapchain::swapchain_context::SwapchainContext;
use crate::resources::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};
use crate::resources::pipeline_layout::pipeline_layout_config::{
    PipelineLayoutConfig, PushConstantRange,
};
use crate::resources::resource_hub::ResourceHub;
use anyhow::Result;
use ash::vk::{AttachmentLoadOp, AttachmentStoreOp, BlendFactor, BlendOp, ColorComponentFlags, CompareOp, CullModeFlags, Extent2D, FrontFace, ImageLayout, Offset2D, Pipeline, PipelineBindPoint, PipelineLayout, PolygonMode, PrimitiveTopology, Rect2D, RenderingAttachmentInfoKHR, RenderingInfoKHR, SampleCountFlags, ShaderStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::render::vulkan::render_pass::collider::collider_push_constants::ColliderPushConstants;
use crate::render::vulkan::resource_context::ResourceContext;
use crate::resources::descriptor_set_layout::descriptor_set_layout_config::DescriptorSetLayoutConfig;

pub struct ColliderRenderPass {
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    buffer_manager: Arc<BufferManager>,
}

impl ColliderRenderPass {
    pub fn create(
        resource_context: &ResourceContext,
        swapchain_context: &SwapchainContext,
        render_context: &RenderContext,
        resource_hub: Arc<ResourceHub>,
    ) -> Result<Self> {
        let pipeline_stages = vec![
            PipelineStageConfig {
                shader_name: String::from("shaders/collider.frag.spv"),
                fn_name: String::from("main"),
                stage: ShaderStageFlags::FRAGMENT,
            },
            PipelineStageConfig {
                shader_name: String::from("shaders/collider.vert.spv"),
                fn_name: String::from("main"),
                stage: ShaderStageFlags::VERTEX,
            },
        ];

        let descriptor_set_layout_config = DescriptorSetLayoutConfig::default();

        let pipeline_layout_config = PipelineLayoutConfig {
            descriptor_set_layout_configs: vec![descriptor_set_layout_config],
            push_constant_ranges: vec![PushConstantRange {
                stage: ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT,
                offset: 0,
                size: size_of::<ColliderPushConstants>() as u32,
            }],
        };

        let pipeline_layout = *resource_hub
            .get_pipeline_layout_provider()
            .get_now(&pipeline_layout_config);

        let pipeline_config = PipelineConfig {
            stages: pipeline_stages,

            color_formats: vec![swapchain_context.format],
            depth_format: Some(render_context.render_targets.depth_image.image_description.format),

            cull_mode: CullModeFlags::NONE,
            polygon_mode: PolygonMode::LINE,
            front_face: FrontFace::COUNTER_CLOCKWISE,
            primitive_topology: PrimitiveTopology::LINE_LIST,

            depth_test: true,
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

impl RenderPass for ColliderRenderPass {
    fn is_enabled(&self) -> bool {
        true
    }

    fn begin_record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()> {
        let depth_image = &render_pass_context
            .render_context
            .render_targets
            .depth_image;

        let color_attachment = RenderingAttachmentInfoKHR::default()
            .image_view(render_pass_context.swapchain_image.image_view)
            .image_layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(AttachmentLoadOp::LOAD)
            .store_op(AttachmentStoreOp::STORE);

        let depth_attachment = RenderingAttachmentInfoKHR::default()
            .image_view(depth_image.image_view)
            .image_layout(ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .load_op(AttachmentLoadOp::LOAD)
            .store_op(AttachmentStoreOp::NONE);

        let color_attachments = vec![color_attachment];

        let rendering_info = RenderingInfoKHR::default()
            .render_area(Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent: Extent2D {
                    width: depth_image.image_description.extent.width,
                    height: depth_image.image_description.extent.height,
                },
            })
            .layer_count(1)
            .color_attachments(&color_attachments)
            .depth_attachment(&depth_attachment);

        render_pass_context.begin_rendering(&rendering_info);

        Ok(())
    }

    fn record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()> {
        render_pass_context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        render_pass_context.set_scissor();
        render_pass_context.set_viewport();

        render_pass_context.push_constants(
            self.pipeline_layout,
            ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT,
            &ColliderPushConstants::create(
                self.buffer_manager.scene_buffer.handle.device_address.unwrap(),
            ),
        );

        render_pass_context.draw_indirect_non_indexed_gpu_scene(
            &self.buffer_manager.collider_indirect_buffer,
            &self.buffer_manager.draw_count_buffer,
        );
        
        Ok(())
    }

    fn end_record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()> {
        render_pass_context.end_rendering();

        Ok(())
    }

    fn destroy(&self) -> Result<()> {
        info!("ColliderRenderPass destroyed");

        Ok(())
    }
}
