use render_graph::VirtualData;
use settings::RenderSettings;
use render_graph::Pass;
use render_graph::FrameContext;
use crate::render::pass_resources::pass_resources::PassResources;
use anyhow::{bail, Result};
use render_snapshot::RenderSnapshot;
use ash::vk::{AccessFlags, CullModeFlags, Format, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PolygonMode, PrimitiveTopology};
use std::sync::Arc;
use tracing::info;
use crate::render::pass::physics_debug::gpu::physics_debug_vertex_gpu::PhysicsDebugVertexGPU;
use gpu::ResourceFactories;
use crate::render::pass::physics_debug::physics_debug_fragment_shader::PhysicsDebugFragmentShader;
use crate::render::push_constants::pipeline_push_constants::PipelinePushConstants;
use crate::render::push_constants::push_constants::PushConstants;
use crate::render::push_constants::vertex::physics_debug_vertex_shader::PhysicsDebugVertexShader;
use render_graph::PassResourceDeclaration;
use render_graph::PrepareScopes;
use render_graph::RecordScopes;
use render_graph::DataResourceScope;
use render_graph::VirtualBuffer;
use render_graph::{ColorTarget, RenderTargets};
use render_graph::VirtualImage;
use resource_residency::ResRef;
use gpu::PipelineLayoutType;
use pipeline_store::PipelineConfig;

pub struct PhysicsDebugPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,
    push_constants: PipelinePushConstants<PhysicsDebugVertexShader, PhysicsDebugFragmentShader>,
    
    render_settings: VirtualData<RenderSettings>,
    render_snapshot: VirtualData<RenderSnapshot>,

    target_image: VirtualImage,

    camera_buffer: VirtualBuffer,
    physics_debug_vertex_buffer: VirtualBuffer,
}

impl PhysicsDebugPass {
    pub fn create(
        resources: &PassResources,
        color_format: Format,
        target_image: VirtualImage,
        physics_debug_vertex_buffer: VirtualBuffer,
        camera_buffer: VirtualBuffer,
        render_snapshot: VirtualData<RenderSnapshot>,
        render_settings: VirtualData<RenderSettings>,
    ) -> Result<Self> {
        let push_constants = PipelinePushConstants::<PhysicsDebugVertexShader, PhysicsDebugFragmentShader>::new();

        let pipeline_config = PipelineConfig {
            label: "physics_debug".to_string(),

            stages: push_constants.stages_layout(),

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
            push_constants,

            render_settings,
            render_snapshot,

            target_image,

            camera_buffer,
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
    
    fn is_enabled(&self, data_scope: &DataResourceScope) -> bool {
        data_scope.get(self.render_settings).collider_rendering.value
    }

    fn prepare_data(
        &self,
        scopes: &mut PrepareScopes,
        _frame_context: &FrameContext,
    ) -> Result<Self::PassData> {
        let render_snapshot = scopes.data.get(self.render_snapshot);

        let physics_debug_vertex_gpu = render_snapshot.debug_lines.iter()
            .flat_map(|physics_debug_line| [
                PhysicsDebugVertexGPU::new(physics_debug_line.start, physics_debug_line.color),
                PhysicsDebugVertexGPU::new(physics_debug_line.end, physics_debug_line.color),
            ]).collect::<Vec<_>>();

        self.physics_debug_vertex_buffer.stage_slice(scopes.buffer, &physics_debug_vertex_gpu)?;

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
                self.camera_buffer,
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
        scopes: &RecordScopes,
        data: Self::PassData,
    ) -> Result<()> {
        if data.physics_debug_vertex_count == 0 {
            return Ok(());
        }

        let physics_debug_buffer = scopes.buffer.get_physical_buffer(self.physics_debug_vertex_buffer);
        let camera_buffer = scopes.buffer.get_physical_buffer(self.camera_buffer);

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        self.push_constants.push(
            context,
            self.pipeline_layout,
            &PushConstants {
                vertex: PhysicsDebugVertexShader::create(
                    camera_buffer.range,
                    physics_debug_buffer.range,
                ),
                fragment: PhysicsDebugFragmentShader,
            },
        );

        context.draw(data.physics_debug_vertex_count as u32);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("PhysicsDebugRenderPass destroyed");

        Ok(())
    }
}
