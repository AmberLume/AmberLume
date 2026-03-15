use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::render_pass::depth::depth_push_constants::DepthPushConstants;
use crate::render::render_pass::render_pass::RenderPass;
use crate::render::render_pass::render_pass_context::RenderPassContext;
use crate::render::render_pass::utils::transition_image_layout;
use crate::render::render_context::RenderContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, AttachmentLoadOp, AttachmentStoreOp, BlendFactor, BlendOp, ClearDepthStencilValue, ClearValue, ColorComponentFlags, CompareOp, CullModeFlags, Extent2D, FrontFace, ImageLayout, Offset2D, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PolygonMode, PrimitiveTopology, Rect2D, RenderingAttachmentInfoKHR, RenderingInfo, SampleCountFlags, ShaderStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::resources::resource_context::ResourceContext;
use crate::resources::dynamic::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::dynamic::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::persistent::persistent_resources::PersistentResources;

pub struct DepthRenderPass {
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    _pipeline_handle: Arc<ResRef>,

    buffer_manager: Arc<BufferManager>,
}

impl DepthRenderPass {
    pub fn create(
        resource_context: &ResourceContext,
        render_context: &RenderContext,
        pipeline_provider: &ResourceProvider<PipelineBackend>,
        persistent_resources: &PersistentResources,
    ) -> Result<Self> {
        let pipeline_stages = vec![
            PipelineStageConfig {
                shader_name: String::from("shaders/depth/depth.frag.spv"),
                fn_name: String::from("main"),
                stage: ShaderStageFlags::FRAGMENT,
            },
            PipelineStageConfig {
                shader_name: String::from("shaders/depth/depth.vert.spv"),
                fn_name: String::from("main"),
                stage: ShaderStageFlags::VERTEX,
            },
        ];

        let pipeline_config = PipelineConfig {
            label: "depth".to_string(),

            stages: pipeline_stages,

            color_formats: vec![],
            depth_format: Some(render_context.transient_resources.depth.image_description.format),

            cull_mode: CullModeFlags::BACK,
            polygon_mode: PolygonMode::FILL,
            front_face: FrontFace::COUNTER_CLOCKWISE,
            primitive_topology: PrimitiveTopology::TRIANGLE_LIST,

            depth_bias_enable: false,
            depth_bias_constant_factor: 0.0,
            depth_bias_slope_factor: 0.0,

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

impl RenderPass for DepthRenderPass {
    fn is_enabled(&self) -> bool {
        true
    }

    fn begin_record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()> {
        let depth_image = &render_pass_context.render_context.transient_resources.depth;
        transition_image_layout(
            &render_pass_context,
            depth_image.image,
            depth_image.image_subresource_range,
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

        let rendering_info = RenderingInfo::default()
            .render_area(Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent: Extent2D {
                    width: depth_image.image_description.extent.width,
                    height: depth_image.image_description.extent.height,
                },
            })
            .layer_count(1)
            .depth_attachment(&depth_attachment);

        render_pass_context.begin_rendering(&rendering_info);

        Ok(())
    }

    fn record_commands(&self, render_pass_context: &RenderPassContext) -> Result<()> {
        render_pass_context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        render_pass_context.set_image_scissor(&render_pass_context.render_context.transient_resources.depth);
        render_pass_context.set_viewport(&render_pass_context.render_context.transient_resources.depth);

        render_pass_context.bind_index_buffer(self.buffer_manager.index_buffer.handle());

        let main_chunk_index = render_pass_context.render_views_layout.get_main_index();
        render_pass_context.push_constants(
            self.pipeline_layout,
            &DepthPushConstants::create(
                self.buffer_manager.scene_buffer.frame(render_pass_context.frame_index).get().device_address(),
                self.buffer_manager.draw_data_buffer.chunk(main_chunk_index).all().device_address(),
                self.buffer_manager.entity_buffer.frame(render_pass_context.frame_index).all().device_address(),
                self.buffer_manager.vertex_buffer.all().device_address(),
            ),
        );
        render_pass_context.draw_indirect_gpu_scene(
            &self.buffer_manager.indirect_buffer.chunk(main_chunk_index),
            &self.buffer_manager.draw_count_buffer.chunk(main_chunk_index),
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
