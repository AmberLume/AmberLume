use render_graph::ReadbackScope;
use render_graph::VirtualData;
use settings::RenderSettings;
use render_graph::Pass;
use render_graph::FrameContext;
use crate::render::pass::pass_resources::PassResources;
use anyhow::{bail, Result};
use render_snapshot::RenderSnapshot;
use ash::vk::{AccessFlags, CullModeFlags, Format, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PolygonMode, PrimitiveTopology};
use std::sync::Arc;
use tracing::info;
use crate::render::frame_data::physics_debug_vertex_gpu::PhysicsDebugVertexGPU;
use gpu::ResourceFactories;
use crate::render::pass::physics_debug::physics_debug_push_constants::PhysicsDebugPushConstants;
use render_graph::PassResourceDeclaration;
use render_graph::ImageResourceScope;
use render_graph::BufferResourceScope;
use render_graph::DataResourceScope;
use render_graph::VirtualBuffer;
use render_graph::{ColorTarget, RenderTargets};
use render_graph::VirtualImage;
use resource_residency::ResRef;
use gpu::PipelineLayoutType;
use pipeline_store::PipelineConfig;
use pipeline_store::PipelineStageConfig;
use crate::resource_manifest::shaders;

pub struct PhysicsDebugPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,
    
    target_image: VirtualImage,

    physics_debug_vertex_buffer: VirtualBuffer,
    scene_buffer: VirtualBuffer,

    render_snapshot: VirtualData<RenderSnapshot>,
    render_settings: VirtualData<RenderSettings>,
}

impl PhysicsDebugPass {
    pub fn create(
        resources: &PassResources,
        color_format: Format,
        target_image: VirtualImage,
        physics_debug_vertex_buffer: VirtualBuffer,
        scene_buffer: VirtualBuffer,
        render_snapshot: VirtualData<RenderSnapshot>,
        render_settings: VirtualData<RenderSettings>,
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

        let _handle = resources.pipeline_provider.acquire_sync(pipeline_config)?;
        let Some(pipeline) = resources.pipeline_provider.with_resource(_handle.id, |pipeline| *pipeline) else {
            bail!("Failed to acquire Pipeline");
        };

        Ok(Self {
            _handle,

            pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            target_image,

            physics_debug_vertex_buffer,
            scene_buffer,

            render_snapshot,
            render_settings,
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
    
    fn is_enabled(&self, data_scope: &DataResourceScope) -> bool {
        data_scope.get(self.render_settings).collider_rendering.value
    }

    fn prepare_data(
        &self,
        data_scope: &mut DataResourceScope,
        buffer_scope: &mut BufferResourceScope,
        _frame_context: &FrameContext,
    ) -> Result<Self::PassData> {
        let render_snapshot = data_scope.get(self.render_snapshot);

        let physics_debug_vertex_gpu = render_snapshot.debug_lines.iter()
            .flat_map(|physics_debug_line| [
                PhysicsDebugVertexGPU::new(physics_debug_line.start, physics_debug_line.color),
                PhysicsDebugVertexGPU::new(physics_debug_line.end, physics_debug_line.color),
            ]).collect::<Vec<_>>();

        self.physics_debug_vertex_buffer.stage_slice(buffer_scope, &physics_debug_vertex_gpu)?;

        Ok(PhysicsDebugRenderPassData {
            physics_debug_vertex_count: physics_debug_vertex_gpu.len(),
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .consume(self.render_snapshot)
            .consume(self.render_settings)
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
            )
            .read_buffer(
                self.scene_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER,
            );
    }

    fn render_targets(&self) -> Option<RenderTargets> {
        Some(RenderTargets {
            color: vec![ColorTarget { image: self.target_image, mip: None, clear: None }],
            depth: None,
            view_mask: 0,
        })
    }

    fn record_commands(
        &self,
        context: &FrameContext,
        _image_scope: &ImageResourceScope,
        buffer_scope: &BufferResourceScope,
        _readback_scope: &ReadbackScope,
        data: Self::PassData,
    ) -> Result<()> {
        if data.physics_debug_vertex_count == 0 {
            return Ok(());
        }

        let physics_debug_buffer = buffer_scope.get_physical_buffer(self.physics_debug_vertex_buffer);
        let scene_buffer = buffer_scope.get_physical_buffer(self.scene_buffer);

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &PhysicsDebugPushConstants::create(
                scene_buffer.range,
                physics_debug_buffer.range,
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
