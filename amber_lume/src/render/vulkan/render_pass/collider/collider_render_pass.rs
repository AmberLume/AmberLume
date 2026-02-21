use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::render_pass::render_pass::RenderPass;
use crate::render::vulkan::render_pass::render_pass_context::RenderPassContext;
use crate::render::vulkan::renderer::render_context::RenderContext;
use crate::render::vulkan::swapchain::swapchain_context::SwapchainContext;
use anyhow::{bail, Result};
use ash::vk::{AttachmentLoadOp, AttachmentStoreOp, BlendFactor, BlendOp, ColorComponentFlags, CompareOp, CullModeFlags, Extent2D, FrontFace, ImageLayout, Offset2D, Pipeline, PipelineBindPoint, PipelineLayout, PolygonMode, PrimitiveTopology, Rect2D, RenderingAttachmentInfoKHR, RenderingInfoKHR, SampleCountFlags, ShaderStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::render::vulkan::render_pass::collider::collider_push_constants::ColliderPushConstants;
use crate::render::vulkan::resource_context::ResourceContext;
use crate::resources::dynamic::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::dynamic::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::persistent::persistent_resources::PersistentResources;

pub struct ColliderRenderPass {
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    _pipeline_handle: Arc<ResRef>,

    buffer_manager: Arc<BufferManager>,
}

impl ColliderRenderPass {
    pub fn create(
        resource_context: &ResourceContext,
        swapchain_context: &SwapchainContext,
        render_context: &RenderContext,
        pipeline_provider: &ResourceProvider<PipelineBackend>,
        persistent_resources: &PersistentResources,
    ) -> Result<Self> {
        let pipeline_stages = vec![
            PipelineStageConfig {
                shader_name: String::from("shaders/collider/collider.frag.spv"),
                fn_name: String::from("main"),
                stage: ShaderStageFlags::FRAGMENT,
            },
            PipelineStageConfig {
                shader_name: String::from("shaders/collider/collider.vert.spv"),
                fn_name: String::from("main"),
                stage: ShaderStageFlags::VERTEX,
            },
        ];

        let pipeline_config = PipelineConfig {
            label: "collider".to_string(),

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

        let extent = render_pass_context.swapchain_image.extent;
        let aspect_ratio = extent.width as f32 / extent.height as f32;

        let projection_matrix = render_pass_context.world_snapshot.camera_stamp.to_view_projection_matrix(aspect_ratio);
        
        render_pass_context.push_constants(
            self.pipeline_layout,
            &ColliderPushConstants::create(
                projection_matrix.to_cols_array_2d(),
                self.buffer_manager.collider_buffer.handle.device_address.unwrap(),
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
