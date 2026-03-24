use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::render_pass::main::main_push_constants::MainPushConstants;
use crate::render::render_pass::render_pass::RenderPass;
use crate::render::render_pass::render_pass_context::RenderPassContext;
use crate::render::render_context::RenderContext;
use crate::render::swapchain::swapchain_context::SwapchainContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, AttachmentLoadOp, AttachmentStoreOp, BlendFactor, BlendOp, ColorComponentFlags, CompareOp, CullModeFlags, FrontFace, ImageLayout, Offset2D, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PolygonMode, PrimitiveTopology, Rect2D, RenderingInfo, SampleCountFlags, ShaderStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::render::render_pass::frame_data_context::FrameDataContext;
use crate::render::render_pass::utils::{extent_3d_to_2d, ImageAttachment};
use crate::render::resources::resource_context::ResourceContext;
use crate::resources::dynamic::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::dynamic::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::persistent::persistent_resources::PersistentResources;

pub struct MainRenderPass {
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    _pipeline_handle: Arc<ResRef>,
    
    buffer_manager: Arc<BufferManager>,
}

impl MainRenderPass {
    pub fn create(
        resource_context: &ResourceContext,
        swapchain_context: &SwapchainContext,
        render_context: &RenderContext,
        pipeline_provider: &ResourceProvider<PipelineBackend>,
        persistent_resources: &PersistentResources,
    ) -> Result<Self> {
        let pipeline_stages = vec![
            PipelineStageConfig {
                shader_name: String::from("shaders/main/main.frag.spv"),
                fn_name: String::from("main"),
                stage: ShaderStageFlags::FRAGMENT,
            },
            PipelineStageConfig {
                shader_name: String::from("shaders/main/main.vert.spv"),
                fn_name: String::from("main"),
                stage: ShaderStageFlags::VERTEX,
            },
        ];

        let pipeline_config = PipelineConfig {
            label: "main".to_string(),
            
            stages: pipeline_stages,

            color_formats: vec![swapchain_context.format],
            depth_format: Some(render_context.transient_resources.depth.image_description.format),

            cull_mode: CullModeFlags::BACK,
            polygon_mode: PolygonMode::FILL,
            front_face: FrontFace::COUNTER_CLOCKWISE,
            primitive_topology: PrimitiveTopology::TRIANGLE_LIST,

            depth_bias_enable: false,
            depth_bias_constant_factor: 0.0,
            depth_bias_slope_factor: 0.0,

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

impl RenderPass for MainRenderPass {
    type RenderPassData = ();

    fn is_enabled(&self) -> bool {
        true
    }

    fn prepare_data(&self, _context: &FrameDataContext) -> Result<Self::RenderPassData> {
        Ok(())
    }

    fn record_commands(&self, context: &RenderPassContext, _data: Self::RenderPassData) -> Result<()> {
        let depth_image = &context.render_context.transient_resources.depth;
        context.transition_image_layout(
            depth_image.image,
            depth_image.image_subresource_range,
            ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
            AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
            PipelineStageFlags::LATE_FRAGMENT_TESTS,
            PipelineStageFlags::EARLY_FRAGMENT_TESTS | PipelineStageFlags::LATE_FRAGMENT_TESTS,
        );

        context.transition_image_layout(
            context.swapchain_image.image,
            context.swapchain_image.image_subresource_range,
            ImageLayout::UNDEFINED,
            ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            AccessFlags::empty(),
            AccessFlags::COLOR_ATTACHMENT_WRITE,
            PipelineStageFlags::TOP_OF_PIPE,
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        );

        let color_attachment = ImageAttachment::from(context.swapchain_image.image_view)
            .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .ops(AttachmentLoadOp::CLEAR, AttachmentStoreOp::STORE)
            .clear_color([0.5, 0.5, 0.5, 1.0]);

        let depth_attachment = ImageAttachment::from(depth_image.image_view)
            .layout(ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .ops(AttachmentLoadOp::LOAD, AttachmentStoreOp::STORE)
            .clear_depth_stencil(1.0, 0);

        let color_attachments = vec![color_attachment.info];

        let rendering_info = RenderingInfo::default()
            .render_area(Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent: extent_3d_to_2d(depth_image.image_description.extent),
            })
            .layer_count(1)
            .color_attachments(&color_attachments)
            .depth_attachment(&depth_attachment.info);

        context.begin_rendering(&rendering_info);

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.set_image_scissor(&context.render_context.transient_resources.depth);
        context.set_viewport(&context.render_context.transient_resources.depth);

        context.bind_index_buffer(self.buffer_manager.index_buffer.as_view());

        let main_render_view_index = context.render_views_layout.get_main_index();
        context.push_constants(
            self.pipeline_layout,
            &MainPushConstants::create(
                self.buffer_manager.scene_buffer.frame(context.frame_index),
                self.buffer_manager.draw_data_buffer.chunk(main_render_view_index),
                self.buffer_manager.vertex_buffer.as_view(),
                self.buffer_manager.entity_buffer.frame(context.frame_index),
                self.buffer_manager.submesh_buffer.as_view(),
                self.buffer_manager.material_buffer.as_view(),
                context.render_context.transient_resources.shadow_mask_descriptor_id,
            ),
        );

        context.draw_indirect_gpu_scene(
            &self.buffer_manager.indirect_buffer.chunk(main_render_view_index),
            &self.buffer_manager.draw_count_buffer.chunk(main_render_view_index),
        );

        context.end_rendering();

        Ok(())
    }

    fn destroy(&self) -> Result<()> {
        info!("MainRenderPass destroyed");

        Ok(())
    }
}
