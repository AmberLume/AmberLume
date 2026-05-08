use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_context::RenderContext;
use crate::render::swapchain::swapchain_context::SwapchainContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, AttachmentLoadOp, AttachmentStoreOp, BlendFactor, BlendOp, ColorComponentFlags, CompareOp, CullModeFlags, FrontFace, ImageLayout, Offset2D, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PolygonMode, PrimitiveTopology, Rect2D, RenderingAttachmentInfoKHR, RenderingInfo, SampleCountFlags, ShaderStageFlags};
use std::sync::Arc;
use arc_swap::ArcSwap;
use tracing::info;
use crate::ids::FrameIndex;
use crate::render::frame_data::physics_debug_vertex_gpu::PhysicsDebugVertexGPU;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::physics_debug::physics_debug_push_constants::PhysicsDebugPushConstants;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::render_graph::resource_registry::resource_registry::ResourceRegistry;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::store::providers::resource_provider::ResourceProvider;
use crate::resources::binding_layout::pipeline_layout_registry::{PipelineLayoutRegistry, PipelineLayoutType};
use crate::resources::store::providers::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::store::providers::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};
use crate::settings::settings::EngineSettings;

pub struct PhysicsDebugPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    settings: Arc<ArcSwap<EngineSettings>>,
    
    swapchain_image: VirtualImage,
    depth_image: VirtualImage,

    physics_debug_vertex_buffer: VirtualBuffer,
}

impl PhysicsDebugPass {
    pub fn create(
        swapchain_context: &SwapchainContext,
        render_context: &RenderContext,
        pipeline_provider: &ResourceProvider<PipelineBackend>,
        pipeline_layout_registry: &PipelineLayoutRegistry,
        settings: Arc<ArcSwap<EngineSettings>>,
        swapchain_image: VirtualImage,
        depth_image: VirtualImage,
        physics_debug_vertex_buffer: VirtualBuffer,
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

            settings,

            swapchain_image,
            depth_image,

            physics_debug_vertex_buffer,
        })
    }
}

pub struct PhysicsDebugRenderPassData {
    physics_debug_vertex_count: usize,
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

    fn prepare_data(
        &self, 
        context: &FrameDataContext,
        resource_registry: &mut ResourceRegistry,
        allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        let physics_debug_vertex_gpu = context.render_snapshot.physics_debug_lines.iter().flat_map(|physics_debug_line| {
            [
                PhysicsDebugVertexGPU::new(physics_debug_line.start, physics_debug_line.color),
                PhysicsDebugVertexGPU::new(physics_debug_line.end, physics_debug_line.color),
            ]
        }).collect::<Vec<_>>();

        self.physics_debug_vertex_buffer.stage_slice(resource_registry, allocator, &physics_debug_vertex_gpu)?;

        Ok(PhysicsDebugRenderPassData {
            physics_debug_vertex_count: physics_debug_vertex_gpu.len(),
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .read_image(
                self.swapchain_image,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_READ,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            )
            .write_image(
                self.swapchain_image,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_WRITE,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            )
            .read_image(
                self.depth_image,
                ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
                PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .write_buffer(
                self.physics_debug_vertex_buffer,
                AccessFlags::HOST_WRITE,
                PipelineStageFlags::HOST,
            )
            .read_buffer(
                self.physics_debug_vertex_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER | PipelineStageFlags::FRAGMENT_SHADER,
            );
    }

    fn record_commands(&self, context: &PassContext, resource_registry: &ResourceRegistry, data: Self::PassData) -> Result<()> {
        if data.physics_debug_vertex_count == 0 {
            return Ok(());
        }
        
        let swapchain_image = resource_registry.get_physical_image(self.swapchain_image);
        let depth_image = resource_registry.get_physical_image(self.depth_image);

        let physics_debug_buffer = resource_registry.get_physical_buffer(self.physics_debug_vertex_buffer);

        let color_attachment = RenderingAttachmentInfoKHR::default()
            .image_view(swapchain_image.image_view)
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
                extent: swapchain_image.extent,
            })
            .layer_count(1)
            .color_attachments(&color_attachments)
            .depth_attachment(&depth_attachment);

        context.begin_rendering(&rendering_info);

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.set_image_scissor(&swapchain_image);
        context.set_viewport(&swapchain_image);
        
        context.push_constants(
            self.pipeline_layout,
            &PhysicsDebugPushConstants::create(
                &context.render_views_layout.main.view_projection,
                physics_debug_buffer,
            ),
        );

        context.draw(data.physics_debug_vertex_count as u32);

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
