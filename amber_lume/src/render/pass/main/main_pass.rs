use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::pass::main::main_push_constants::MainPushConstants;
use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_context::RenderContext;
use crate::render::swapchain::swapchain_context::SwapchainContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, AttachmentLoadOp, AttachmentStoreOp, BlendFactor, BlendOp, ColorComponentFlags, CompareOp, CullModeFlags, FrontFace, ImageLayout, Offset2D, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PolygonMode, PrimitiveTopology, Rect2D, RenderingInfo, SampleCountFlags, ShaderStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::ids::FrameIndex;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::utils::ImageAttachment;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::render_graph::resource_registry::resource_registry::ResourceRegistry;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::render::resources::resource_context::ResourceContext;
use crate::resources::dynamic::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::dynamic::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::pipeline_layout_registry::{PipelineLayoutRegistry, PipelineLayoutType};

pub struct MainPass {
    _handle: Arc<ResRef>,
    
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,
    
    buffer_manager: Arc<BufferManager>,

    swapchain: VirtualImage,
    depth: VirtualImage,
    shadow_mask: VirtualImage,
}

impl MainPass {
    pub fn create(
        resource_context: &ResourceContext,
        swapchain_context: &SwapchainContext,
        render_context: &RenderContext,
        pipeline_provider: &ResourceProvider<PipelineBackend>,
        pipeline_layout_registry: &PipelineLayoutRegistry,
        swapchain: VirtualImage,
        depth: VirtualImage,
        shadow_mask: VirtualImage,
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
            depth_format: Some(render_context.depth_format),

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

        let _handle = pipeline_provider.acquire_sync(pipeline_config);
        let Some(pipeline) = pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire Pipeline");
        };

        Ok(Self {
            _handle,
            
            pipeline: *pipeline,
            pipeline_layout: pipeline_layout_registry.get(PipelineLayoutType::General),
            
            buffer_manager: resource_context.buffer_manager.clone(),

            swapchain,
            depth,
            shadow_mask,
        })
    }
}

impl Pass for MainPass {
    type PassData = ();
    type Statistics = ();

    fn name(&self) -> String {
        String::from("main")
    }
    
    fn is_enabled(&self) -> bool {
        true
    }

    fn prepare_data(&self, _context: &FrameDataContext) -> Result<Self::PassData> {
        Ok(())
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .read(
                self.depth,
                ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
                PipelineStageFlags::EARLY_FRAGMENT_TESTS | PipelineStageFlags::LATE_FRAGMENT_TESTS,
            )
            .read(
                self.shadow_mask,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            )
            .write(
                self.swapchain,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_WRITE,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            );
    }
    
    fn record_commands(&self, context: &PassContext, resource_registry: &ResourceRegistry, _data: Self::PassData) -> Result<()> {
        let swapchain = resource_registry.get(self.swapchain);
        let depth = resource_registry.get(self.depth);
        let shadow_mask = resource_registry.get(self.shadow_mask);

        let color_attachment = ImageAttachment::from(swapchain.image_view)
            .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .ops(AttachmentLoadOp::CLEAR, AttachmentStoreOp::STORE)
            .clear_color([0.5, 0.5, 0.5, 1.0]);

        let depth_attachment = ImageAttachment::from(depth.image_view)
            .layout(ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .ops(AttachmentLoadOp::LOAD, AttachmentStoreOp::STORE)
            .clear_depth_stencil(1.0, 0);

        let color_attachments = vec![color_attachment.info];

        let rendering_info = RenderingInfo::default()
            .render_area(Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent: swapchain.extent,
            })
            .layer_count(1)
            .color_attachments(&color_attachments)
            .depth_attachment(&depth_attachment.info);

        context.begin_rendering(&rendering_info);

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.set_image_scissor(&swapchain);
        context.set_viewport(&swapchain);

        context.bind_index_buffer();

        let main_render_view_index = context.render_views_layout.get_main_index();
        context.push_constants(
            self.pipeline_layout,
            &MainPushConstants::create(
                self.buffer_manager.scene_buffer.frame(context.frame_index),
                self.buffer_manager.draw_data_buffer.chunk(main_render_view_index),
                context.resource_buffers.vertex_buffer,
                self.buffer_manager.entity_buffer.frame(context.frame_index),
                context.resource_buffers.submesh_buffer,
                context.resource_buffers.material_buffer,
                shadow_mask.descriptor_id.unwrap(),
            ),
        );

        context.draw_indirect_gpu_scene(
            &self.buffer_manager.indirect_buffer.chunk(main_render_view_index),
            &self.buffer_manager.draw_count_buffer.chunk(main_render_view_index),
        );

        context.end_rendering();

        Ok(())
    }

    fn statistics(&self, _frame_index: FrameIndex) -> Self::Statistics {
        ()
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("MainRenderPass destroyed");

        Ok(())
    }
}
