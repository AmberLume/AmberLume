use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::render_pass::render_pass::RenderPass;
use crate::render::render_pass::render_pass_context::RenderPassContext;
use crate::render::swapchain::swapchain_context::SwapchainContext;
use anyhow::{bail, Result};
use ash::vk::{AttachmentLoadOp, AttachmentStoreOp, BlendFactor, BlendOp, ColorComponentFlags, CompareOp, CullModeFlags, FrontFace, ImageLayout, Offset2D, Pipeline, PipelineBindPoint, PipelineLayout, PolygonMode, PrimitiveTopology, Rect2D, RenderingAttachmentInfoKHR, RenderingInfo, SampleCountFlags, ShaderStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::render::render_pass::frame_data_context::FrameDataContext;
use crate::render::render_pass::ui::ui_push_constants::UiPushConstants;
use crate::render::render_pass::ui::ui_snapshot::UiDrawLayer;
use crate::render::resources::resource_context::ResourceContext;
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

pub struct UiRenderPassData {
    ui_draw_layers: Vec<UiDrawLayer>,
}

impl RenderPass for UiRenderPass {
    type RenderPassData = UiRenderPassData;
    
    fn is_enabled(&self) -> bool {
        true
    }

    fn prepare_data(&self, context: &FrameDataContext) -> Result<Self::RenderPassData> {
        let ui_draw_layers = context.ui_snapshot.draw_layers.iter().map(|l| l.clone()).collect::<Vec<_>>();
        
        Ok(UiRenderPassData {
            ui_draw_layers,
        })
    }

    fn record_commands(&self, context: &RenderPassContext, data: Self::RenderPassData) -> Result<()> {
        let color_attachment = RenderingAttachmentInfoKHR::default()
            .image_view(context.swapchain_image.image_view)
            .image_layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(AttachmentLoadOp::LOAD)
            .store_op(AttachmentStoreOp::STORE);

        let color_attachments = vec![color_attachment];

        let rendering_info = RenderingInfo::default()
            .render_area(Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent: context.swapchain_image.extent,
            })
            .layer_count(1)
            .color_attachments(&color_attachments);

        context.begin_rendering(&rendering_info);
        
        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.set_viewport(&context.render_context.transient_resources.depth);

        context.bind_index_buffer(self.buffer_manager.ui_index_buffer.frame(context.frame_index));

        data.ui_draw_layers.iter().for_each(|draw_layer| {
            draw_layer.draw_calls.iter().for_each(|draw_call| {
                if let Some(clip_area) = &draw_call.clip {
                    context.set_area_scissor(&clip_area);
                } else {
                    context.set_image_scissor(&context.render_context.transient_resources.depth);
                }

                context.push_constants(
                    self.pipeline_layout,
                    &UiPushConstants::create(
                        self.buffer_manager.ui_vertex_buffer.frame(context.frame_index),
                        draw_call.texture_index,
                        draw_call.render_mode as u32,
                    ),
                );

                context.draw_indexed(
                    draw_call.index_count,
                    draw_call.index_offset,
                    draw_call.vertex_offset,
                );
            });
        });

        context.end_rendering();
        
        Ok(())
    }

    fn destroy(self) -> Result<()> {
        info!("MainRenderPass destroyed");

        Ok(())
    }
}
