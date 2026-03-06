use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::render_pass::render_pass::RenderPass;
use crate::render::vulkan::render_pass::render_pass_context::RenderPassContext;
use crate::render::vulkan::render_pass::utils::transition_image_layout;
use crate::render::vulkan::swapchain::swapchain_context::SwapchainContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, AttachmentLoadOp, AttachmentStoreOp, BlendFactor, BlendOp, ColorComponentFlags, CompareOp, CullModeFlags, FrontFace, ImageLayout, Offset2D, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PolygonMode, PrimitiveTopology, Rect2D, RenderingAttachmentInfoKHR, RenderingInfo, SampleCountFlags, ShaderStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::render::vulkan::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::vulkan::render_pass::ui_render_pass::ui_push_constants::UiPushConstants;
use crate::render::vulkan::resource_context::ResourceContext;
use crate::resources::dynamic::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::dynamic::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::persistent::persistent_resources::PersistentResources;

pub struct UiRenderPass {
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    _pipeline_handle: Arc<ResRef>,

    buffer_manager: Arc<BufferManager>,
}

impl UiRenderPass {
    pub fn create(
        resource_context: &ResourceContext,
        swapchain_context: &SwapchainContext,
        pipeline_provider: &ResourceProvider<PipelineBackend>,
        persistent_resources: &PersistentResources,
    ) -> Result<Self> {
        let color_format = swapchain_context.format;

        let pipeline_stages = vec![
            PipelineStageConfig {
                shader_name: String::from("shaders/yakui/yakui.frag.spv"),
                fn_name: String::from("main"),
                stage: ShaderStageFlags::FRAGMENT,
            },
            PipelineStageConfig {
                shader_name: String::from("shaders/yakui/yakui.vert.spv"),
                fn_name: String::from("main"),
                stage: ShaderStageFlags::VERTEX,
            },
        ];

        let pipeline_config = PipelineConfig {
            label: "ui".to_string(),
            
            stages: pipeline_stages,

            color_formats: vec![color_format],
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
        };

        let pipeline_handle = pipeline_provider.acquire_sync(pipeline_config);
        let Some(pipeline) = pipeline_provider.get_resource(pipeline_handle.id) else {
            bail!("Failed to acquire Pipeline");
        };

        Ok(Self {
            pipeline,
            pipeline_layout: persistent_resources.pipeline_layouts.global,

            _pipeline_handle: pipeline_handle,

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
            render_pass_context.swapchain_image.image,
            render_pass_context.swapchain_image.image_subresource_range,
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

        let rendering_info = RenderingInfo::default()
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

        render_pass_context.set_scissor(&render_pass_context.render_context.transient_resources.depth);
        render_pass_context.set_viewport(&render_pass_context.render_context.transient_resources.depth);

        render_pass_context.bind_index_buffer(self.buffer_manager.ui_index_buffer.handle());

        render_pass_context.ui_snapshot.draw_calls.iter().for_each(|call| {
            render_pass_context.push_constants(
                self.pipeline_layout,
                &UiPushConstants::create(
                    self.buffer_manager.ui_vertex_buffer.frame(render_pass_context.frame_index).at(0).device_address(),
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
