use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::pass::depth::depth_push_constants::DepthPushConstants;
use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_context::RenderContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, AttachmentLoadOp, AttachmentStoreOp, BlendFactor, BlendOp, ClearDepthStencilValue, ClearValue, ColorComponentFlags, CompareOp, CullModeFlags, FrontFace, ImageLayout, Offset2D, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PolygonMode, PrimitiveTopology, Rect2D, RenderingAttachmentInfoKHR, RenderingInfo, SampleCountFlags, ShaderStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::ids::FrameIndex;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::render_graph::resource_registry::resource_registry::ResourceRegistry;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::render::resources::resource_context::ResourceContext;
use crate::resources::dynamic::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::dynamic::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::pipeline_layout_registry::{PipelineLayoutRegistry, PipelineLayoutType};

pub struct DepthPass {
    _handle: Arc<ResRef>,
    
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    buffer_manager: Arc<BufferManager>,

    depth: VirtualImage,
}

impl DepthPass {
    pub fn create(
        resource_context: &ResourceContext,
        render_context: &RenderContext,
        pipeline_provider: &ResourceProvider<PipelineBackend>,
        pipeline_layout_registry: &PipelineLayoutRegistry,
        depth: VirtualImage,
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
            depth_format: Some(render_context.transient_resources.depth_format),

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

        let _handle = pipeline_provider.acquire_sync(pipeline_config);
        let Some(pipeline) = pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire Pipeline");
        };

        Ok(Self {
            _handle,
            
            pipeline: *pipeline,
            pipeline_layout: pipeline_layout_registry.get(PipelineLayoutType::General),

            buffer_manager: resource_context.buffer_manager.clone(),

            depth,
        })
    }
}

impl Pass for DepthPass {
    type PassData = ();
    type Statistics = ();

    fn name(&self) -> String {
        String::from("depth")
    }
    
    fn is_enabled(&self) -> bool {
        true
    }

    fn prepare_data(&self, _context: &FrameDataContext) -> Result<Self::PassData> {
        Ok(())
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .image(
                self.depth,
                ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                PipelineStageFlags::EARLY_FRAGMENT_TESTS | PipelineStageFlags::LATE_FRAGMENT_TESTS,
            );
    }

    fn record_commands(&self, context: &PassContext, resource_registry: &ResourceRegistry, _data: Self::PassData) -> Result<()> {
        let depth = resource_registry.get(self.depth);
    
        let depth_attachment = RenderingAttachmentInfoKHR::default()
            .image_view(depth.image_view)
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
                extent: depth.extent,
            })
            .layer_count(1)
            .depth_attachment(&depth_attachment);

        context.begin_rendering(&rendering_info);

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.set_image_scissor(&depth);
        context.set_viewport(&depth);

        context.bind_index_buffer();

        let main_chunk_index = context.render_views_layout.get_main_index();
        context.push_constants(
            self.pipeline_layout,
            &DepthPushConstants::create(
                self.buffer_manager.scene_buffer.frame(context.frame_index),
                self.buffer_manager.draw_data_buffer.chunk(main_chunk_index),
                self.buffer_manager.entity_buffer.frame(context.frame_index),
                context.resource_buffers.vertex_buffer,
            ),
        );
        context.draw_indirect_gpu_scene(
            &self.buffer_manager.indirect_buffer.chunk(main_chunk_index),
            &self.buffer_manager.draw_count_buffer.chunk(main_chunk_index),
        );

        context.end_rendering();

        Ok(())
    }

    fn statistics(&self, _frame_index: FrameIndex) -> Self::Statistics {
        ()
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("DepthRenderPass destroyed");

        Ok(())
    }
}
