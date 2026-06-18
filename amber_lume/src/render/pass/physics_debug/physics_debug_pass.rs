use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, CullModeFlags, Format, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PolygonMode, PrimitiveTopology};
use std::sync::Arc;
use arc_swap::ArcSwap;
use tracing::info;
use crate::render::frame_data::physics_debug_vertex_gpu::PhysicsDebugVertexGPU;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::physics_debug::physics_debug_push_constants::PhysicsDebugPushConstants;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::render::render_graph::virtual_image::render_targets::{ColorTarget, RenderTargets};
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::store::providers::resource_provider::ResourceProvider;
use crate::resources::binding_layout::pipeline_layout_registry::{PipelineLayoutRegistry, PipelineLayoutType};
use crate::resources::store::providers::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::store::providers::pipeline::pipeline_config::{PipelineConfig, PipelineStageConfig};
use crate::resources::resource_manifest::shaders;
use crate::settings::settings::EngineSettings;

pub struct PhysicsDebugPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    settings: Arc<ArcSwap<EngineSettings>>,
    
    target_image: VirtualImage,

    physics_debug_vertex_buffer: VirtualBuffer,
}

impl PhysicsDebugPass {
    pub fn create(
        color_format: Format,
        pipeline_provider: &ResourceProvider<PipelineBackend>,
        pipeline_layout_registry: &PipelineLayoutRegistry,
        settings: Arc<ArcSwap<EngineSettings>>,
        target_image: VirtualImage,
        physics_debug_vertex_buffer: VirtualBuffer,
    ) -> Result<Self> {
        let pipeline_stages = vec![
            PipelineStageConfig::fragment(shaders::PHYSICS_DEBUG_FRAG),
            PipelineStageConfig::vertex(shaders::PHYSICS_DEBUG_VERT),
        ];

        let pipeline_config = PipelineConfig {
            label: "physics_debug".to_string(),

            stages: pipeline_stages,

            color_formats: vec![color_format],

            cull_mode: CullModeFlags::BACK,
            polygon_mode: PolygonMode::LINE,
            primitive_topology: PrimitiveTopology::LINE_LIST,

            ..PipelineConfig::fullscreen()
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

            target_image,

            physics_debug_vertex_buffer,
        })
    }
}

pub struct PhysicsDebugRenderPassData {
    physics_debug_vertex_count: usize,
}

impl Pass for PhysicsDebugPass {
    type PassData = PhysicsDebugRenderPassData;

    fn name(&self) -> String {
        String::from("physics_debug")
    }
    
    fn is_enabled(&self) -> bool {
        self.settings.load().debug.collider_rendering_enabled.value
    }

    fn prepare_data(
        &self, 
        context: &FrameDataContext,
        buffer_scope: &mut BufferResourceScope,
        allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        let physics_debug_vertex_gpu = context.render_snapshot.physics_debug_lines.iter().flat_map(|physics_debug_line| {
            [
                PhysicsDebugVertexGPU::new(physics_debug_line.start, physics_debug_line.color),
                PhysicsDebugVertexGPU::new(physics_debug_line.end, physics_debug_line.color),
            ]
        }).collect::<Vec<_>>();

        self.physics_debug_vertex_buffer.stage_slice(buffer_scope, allocator, &physics_debug_vertex_gpu)?;

        Ok(PhysicsDebugRenderPassData {
            physics_debug_vertex_count: physics_debug_vertex_gpu.len(),
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .read_image(
                self.target_image,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_READ,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            )
            .write_image(
                self.target_image,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_WRITE,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
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

    fn render_targets(&self) -> Option<RenderTargets> {
        Some(RenderTargets {
            color: vec![ColorTarget { image: self.target_image, mip: None, clear: None }],
            depth: None,
            view_mask: 0,
        })
    }

    fn record_commands(&self, context: &PassContext, _image_scope: &ImageResourceScope, buffer_scope: &BufferResourceScope, data: Self::PassData) -> Result<()> {
        if data.physics_debug_vertex_count == 0 {
            return Ok(());
        }

        let physics_debug_buffer = buffer_scope.get_physical_buffer(self.physics_debug_vertex_buffer);

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &PhysicsDebugPushConstants::create(
                &context.render_views_layout.main.view_projection,
                physics_debug_buffer,
            ),
        );

        context.draw(data.physics_debug_vertex_count as u32);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("PhysicsDebugRenderPass destroyed");

        Ok(())
    }
}
