use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::render_pass::render_pass::RenderPass;
use crate::render::render_pass::render_pass_context::RenderPassContext;
use crate::render::swapchain::swapchain_context::SwapchainContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, AttachmentLoadOp, AttachmentStoreOp, BlendFactor, BlendOp, ColorComponentFlags, CompareOp, CullModeFlags, DependencyFlags, FrontFace, ImageLayout, Offset2D, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PolygonMode, PrimitiveTopology, Rect2D, RenderingAttachmentInfoKHR, RenderingInfo, SampleCountFlags, ShaderStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::ids::SliceIndex;
use crate::render::buffer::typed::ui_vertex_buffer::UiVertex;
use crate::render::render_pass::frame_data_context::FrameDataContext;
use crate::render::render_pass::ui::ui_push_constants::UiPushConstants;
use crate::render::render_pass::ui::ui_snapshot::UiDrawLayer;
use crate::render::resources::resource_context::ResourceContext;
use crate::resources::dynamic::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::dynamic::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::pipeline_layout_registry::{PipelineLayoutRegistry, PipelineLayoutType};

pub struct UiRenderPass {
    _handle: Arc<ResRef>,
    
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    buffer_manager: Arc<BufferManager>,
}

impl UiRenderPass {
    pub fn create(
        resource_context: &ResourceContext,
        swapchain_context: &SwapchainContext,
        pipeline_provider: &ResourceProvider<PipelineBackend>,
        pipeline_layout_registry: &PipelineLayoutRegistry,
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

        let _handle = pipeline_provider.acquire_sync(pipeline_config);
        let Some(pipeline) = pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire Pipeline");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: pipeline_layout_registry.get(PipelineLayoutType::General),

            buffer_manager: resource_context.buffer_manager.clone(),
        })
    }
}

pub struct UiRenderPassData {
    indices: Vec<u32>,
    vertices: Vec<UiVertex>,

    ui_draw_layers: Vec<UiDrawLayer>,
}

impl RenderPass for UiRenderPass {
    type RenderPassData = UiRenderPassData;
    
    fn is_enabled(&self) -> bool {
        true
    }

    fn prepare_data(&self, context: &FrameDataContext) -> Result<Self::RenderPassData> {
        Ok(UiRenderPassData {
            indices: context.ui_snapshot.indices.clone(),
            vertices: context.ui_snapshot.vertices.clone(),

            ui_draw_layers: context.ui_snapshot.draw_layers.clone(),
        })
    }

    fn record_commands(&self, context: &RenderPassContext, data: Self::RenderPassData) -> Result<()> {
        let indices_barrier = self.buffer_manager.ui_index_buffer
            .frame(context.frame_index)
            .slice_at(SliceIndex::ZERO)
            .stage(&data.indices, AccessFlags::SHADER_READ)?;
        let vertices_barrier = self.buffer_manager.ui_vertex_buffer
            .frame(context.frame_index)
            .slice_at(SliceIndex::ZERO)
            .stage(&data.vertices, AccessFlags::SHADER_READ)?;

        let color_attachment = RenderingAttachmentInfoKHR::default()
            .image_view(context.swapchain_image.image_view)
            .image_layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(AttachmentLoadOp::LOAD)
            .store_op(AttachmentStoreOp::STORE);

        context.pipeline_barrier(
            PipelineStageFlags::HOST,
            PipelineStageFlags::VERTEX_SHADER,
            DependencyFlags::empty(),
            &[],
            &[
                indices_barrier,
                vertices_barrier,
            ],
            &[],
        );

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

        context.set_viewport(context.swapchain_image.extent);

        context.bind_index_buffer(self.buffer_manager.ui_index_buffer.frame(context.frame_index));

        data.ui_draw_layers.iter().for_each(|draw_layer| {
            draw_layer.draw_calls.iter().for_each(|draw_call| {
                if let Some(clip_area) = &draw_call.clip {
                    context.set_area_scissor(&clip_area);
                } else {
                    context.set_image_scissor(context.swapchain_image.extent);
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
