use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::render_pass::render_pass::RenderPass;
use crate::render::render_pass::render_pass_context::RenderPassContext;
use crate::render::render_context::RenderContext;
use crate::render::swapchain::swapchain_context::SwapchainContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, AttachmentLoadOp, AttachmentStoreOp, BlendFactor, BlendOp, ColorComponentFlags, CompareOp, CullModeFlags, DependencyFlags, Extent2D, FrontFace, ImageLayout, Offset2D, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PolygonMode, PrimitiveTopology, Rect2D, RenderingAttachmentInfoKHR, RenderingInfo, SampleCountFlags, ShaderStageFlags};
use std::sync::Arc;
use arc_swap::ArcSwap;
use tracing::info;
use crate::ids::SliceIndex;
use crate::render::buffer::typed::physics_debug_vertex_buffer::PhysicsDebugVertexGpuData;
use crate::render::render_pass::frame_data_context::FrameDataContext;
use crate::render::render_pass::physics_debug::physics_debug_push_constants::PhysicsDebugPushConstants;
use crate::render::resources::resource_context::ResourceContext;
use crate::resources::dynamic::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::dynamic::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::persistent::persistent_resources::PersistentResources;
use crate::settings::settings::EngineSettings;

pub struct PhysicsDebugRenderPass {
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    _pipeline_handle: Arc<ResRef>,
    
    buffer_manager: Arc<BufferManager>,

    settings: Arc<ArcSwap<EngineSettings>>,
}

impl PhysicsDebugRenderPass {
    pub fn create(
        resource_context: &ResourceContext,
        swapchain_context: &SwapchainContext,
        render_context: &RenderContext,
        pipeline_provider: &ResourceProvider<PipelineBackend>,
        persistent_resources: &PersistentResources,
        settings: Arc<ArcSwap<EngineSettings>>,
    ) -> Result<Self> {
        let pipeline_stages = vec![
            PipelineStageConfig {
                shader_name: String::from("shaders/physics_debug/physics_debug.frag.spv"),
                fn_name: String::from("main"),
                stage: ShaderStageFlags::FRAGMENT,
            },
            PipelineStageConfig {
                shader_name: String::from("shaders/physics_debug/physics_debug.vert.spv"),
                fn_name: String::from("main"),
                stage: ShaderStageFlags::VERTEX,
            },
        ];

        let pipeline_config = PipelineConfig {
            label: "physics_debug".to_string(),
            
            stages: pipeline_stages,

            color_formats: vec![swapchain_context.format],
            depth_format: Some(render_context.transient_resources.depth.image_description.format),

            cull_mode: CullModeFlags::BACK,
            polygon_mode: PolygonMode::LINE,
            front_face: FrontFace::COUNTER_CLOCKWISE,
            primitive_topology: PrimitiveTopology::LINE_LIST,

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

            settings,
        })
    }
}

pub struct PhysicsDebugRenderPassData {
    physics_debug_vertex_gpu_data: Vec<PhysicsDebugVertexGpuData>,
}

impl RenderPass for PhysicsDebugRenderPass {
    type RenderPassData = PhysicsDebugRenderPassData;

    fn is_enabled(&self) -> bool {
        self.settings.load().debug.collider_rendering_enabled.get()
    }

    fn prepare_data(&self, context: &FrameDataContext) -> Result<Self::RenderPassData> {
        let physics_debug_vertex_gpu_data = context.world_snapshot.physics_debug_lines.iter().flat_map(|physics_debug_line| {
            [
                PhysicsDebugVertexGpuData::new(physics_debug_line.start, physics_debug_line.color),
                PhysicsDebugVertexGpuData::new(physics_debug_line.end, physics_debug_line.color),
            ]
        }).collect::<Vec<_>>();

        Ok(PhysicsDebugRenderPassData {
            physics_debug_vertex_gpu_data,
        })
    }

    fn record_commands(&self, context: &RenderPassContext, data: Self::RenderPassData) -> Result<()> {
        let depth_image = &context.render_context.transient_resources.depth;

        context.transition_image_layout(
            context.swapchain_image.image,
            context.swapchain_image.image_subresource_range,
            ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            AccessFlags::COLOR_ATTACHMENT_WRITE,
            AccessFlags::COLOR_ATTACHMENT_WRITE,
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        );

        let color_attachment = RenderingAttachmentInfoKHR::default()
            .image_view(context.swapchain_image.image_view)
            .image_layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(AttachmentLoadOp::LOAD)
            .store_op(AttachmentStoreOp::STORE);

        let depth_attachment = RenderingAttachmentInfoKHR::default()
            .image_view(depth_image.image_view)
            .image_layout(ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .load_op(AttachmentLoadOp::LOAD)
            .store_op(AttachmentStoreOp::STORE);

        let color_attachments = vec![color_attachment];

        let rendering_info = RenderingInfo::default()
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

        context.begin_rendering(&rendering_info);

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        let physics_debug_vertex_barrier = self.buffer_manager.physics_debug_buffer
            .frame(context.frame_index)
            .slice_at(SliceIndex::ZERO)
            .stage(&data.physics_debug_vertex_gpu_data, AccessFlags::TRANSFER_READ)?;

        context.set_image_scissor(&context.render_context.transient_resources.depth);
        context.set_viewport(&context.render_context.transient_resources.depth);

        context.pipeline_barrier(
            PipelineStageFlags::HOST,
            PipelineStageFlags::TRANSFER,
            DependencyFlags::empty(),
            &[],
            &[
                physics_debug_vertex_barrier,
            ],
            &[],
        );
        
        context.push_constants(
            self.pipeline_layout,
            &PhysicsDebugPushConstants::create(
                context.render_views_layout.main.projection_view.to_cols_array_2d(),
                self.buffer_manager.physics_debug_buffer.frame(context.frame_index),
            ),
        );

        context.draw(data.physics_debug_vertex_gpu_data.len() as u32);

        context.end_rendering();

        Ok(())
    }

    fn destroy(self) -> Result<()> {
        info!("PhysicsDebugRenderPass destroyed");

        Ok(())
    }
}
