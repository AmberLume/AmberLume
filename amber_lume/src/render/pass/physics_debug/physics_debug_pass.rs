use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_context::RenderContext;
use crate::render::swapchain::swapchain_context::SwapchainContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, AttachmentLoadOp, AttachmentStoreOp, BlendFactor, BlendOp, ColorComponentFlags, CompareOp, CullModeFlags, DependencyFlags, FrontFace, ImageLayout, Offset2D, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PolygonMode, PrimitiveTopology, Rect2D, RenderingAttachmentInfoKHR, RenderingInfo, SampleCountFlags, ShaderStageFlags};
use std::sync::Arc;
use arc_swap::ArcSwap;
use tracing::info;
use crate::ids::{FrameIndex, SliceIndex};
use crate::render::buffer::typed::physics_debug_vertex_buffer::PhysicsDebugVertexGPU;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::physics_debug::physics_debug_push_constants::PhysicsDebugPushConstants;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::render_graph::resource_registry::resource_registry::ResourceRegistry;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::render::resources::resource_context::ResourceContext;
use crate::resources::dynamic::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::dynamic::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::binding_layout::pipeline_layout_registry::{PipelineLayoutRegistry, PipelineLayoutType};
use crate::settings::settings::EngineSettings;

pub struct PhysicsDebugPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    buffer_manager: Arc<BufferManager>,

    settings: Arc<ArcSwap<EngineSettings>>,
    
    swapchain: VirtualImage,
    depth: VirtualImage,
}

impl PhysicsDebugPass {
    pub fn create(
        resource_context: &ResourceContext,
        swapchain_context: &SwapchainContext,
        render_context: &RenderContext,
        pipeline_provider: &ResourceProvider<PipelineBackend>,
        pipeline_layout_registry: &PipelineLayoutRegistry,
        settings: Arc<ArcSwap<EngineSettings>>,
        swapchain: VirtualImage,
        depth: VirtualImage,
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
            depth_format: Some(render_context.depth_format),

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

        let _handle = pipeline_provider.acquire_sync(pipeline_config);
        let Some(pipeline) = pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire Pipeline");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: pipeline_layout_registry.get(PipelineLayoutType::General),

            buffer_manager: resource_context.buffer_manager.clone(),

            settings,
            
            swapchain,
            depth,
        })
    }
}

pub struct PhysicsDebugRenderPassData {
    physics_debug_vertex_gpu: Vec<PhysicsDebugVertexGPU>,
}

impl Pass for PhysicsDebugPass {
    type PassData = PhysicsDebugRenderPassData;
    type Statistics = ();

    fn name(&self) -> String {
        String::from("physics_debug")
    }
    
    fn is_enabled(&self) -> bool {
        self.settings.load().debug.collider_rendering_enabled.get()
    }

    fn prepare_data(&self, context: &FrameDataContext) -> Result<Self::PassData> {
        let physics_debug_vertex_gpu = context.render_snapshot.physics_debug_lines.iter().flat_map(|physics_debug_line| {
            [
                PhysicsDebugVertexGPU::new(physics_debug_line.start, physics_debug_line.color),
                PhysicsDebugVertexGPU::new(physics_debug_line.end, physics_debug_line.color),
            ]
        }).collect::<Vec<_>>();

        Ok(PhysicsDebugRenderPassData {
            physics_debug_vertex_gpu,
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .read(
                self.swapchain,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_READ,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            )
            .write(
                self.swapchain,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_WRITE,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            )
            .read(
                self.depth,
                ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
                PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            );
    }

    fn record_commands(&self, context: &PassContext, resource_registry: &ResourceRegistry, data: Self::PassData) -> Result<()> {
        if data.physics_debug_vertex_gpu.is_empty() {
            return Ok(());
        }
        
        let swapchain = resource_registry.get(self.swapchain);
        let depth = resource_registry.get(self.depth);
        
        let color_attachment = RenderingAttachmentInfoKHR::default()
            .image_view(swapchain.image_view)
            .image_layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(AttachmentLoadOp::LOAD)
            .store_op(AttachmentStoreOp::STORE);

        let depth_attachment = RenderingAttachmentInfoKHR::default()
            .image_view(depth.image_view)
            .image_layout(ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .load_op(AttachmentLoadOp::LOAD)
            .store_op(AttachmentStoreOp::STORE);

        let color_attachments = vec![color_attachment];

        let rendering_info = RenderingInfo::default()
            .render_area(Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent: swapchain.extent,
            })
            .layer_count(1)
            .color_attachments(&color_attachments)
            .depth_attachment(&depth_attachment);

        let physics_debug_vertex_barrier = self.buffer_manager.physics_debug_buffer
            .frame(context.frame_index)
            .slice_at(SliceIndex::ZERO)
            .stage(&data.physics_debug_vertex_gpu, AccessFlags::VERTEX_ATTRIBUTE_READ)?;

        context.pipeline_barrier(
            PipelineStageFlags::HOST,
            PipelineStageFlags::VERTEX_INPUT,
            DependencyFlags::empty(),
            &[],
            &[
                physics_debug_vertex_barrier,
            ],
            &[],
        );

        context.begin_rendering(&rendering_info);

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.set_image_scissor(&swapchain);
        context.set_viewport(&swapchain);
        
        context.push_constants(
            self.pipeline_layout,
            &PhysicsDebugPushConstants::create(
                &context.render_views_layout.main.view_projection,
                self.buffer_manager.physics_debug_buffer.frame(context.frame_index),
            ),
        );

        context.draw(data.physics_debug_vertex_gpu.len() as u32);

        context.end_rendering();

        Ok(())
    }

    fn statistics(&self, _frame_index: FrameIndex) -> Self::Statistics {
        ()
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("PhysicsDebugRenderPass destroyed");

        Ok(())
    }
}
