use render_graph::ReadbackScope;
use render_graph::VirtualBuffer;
use render_graph::Pass;
use render_graph::FrameContext;
use crate::render::pass::pass_resources::PassResources;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, CullModeFlags, Format, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::sync::Arc;
use tracing::info;
use gpu::ResourceFactories;
use crate::render::pass::environment::environment_push_constants::EnvironmentPushConstants;
use render_graph::PassResourceDeclaration;
use render_graph::ImageResourceScope;
use render_graph::BufferResourceScope;
use render_graph::DataResourceScope;
use render_graph::HeapAllocator;
use render_graph::{ClearColor, ColorTarget, DepthTarget, RenderTargets};
use render_graph::VirtualImage;
use resource_residency::ResRef;
use gpu::PipelineLayoutType;
use pipeline_store::PipelineConfig;
use pipeline_store::PipelineStageConfig;
use crate::resource_manifest::shaders;

pub struct EnvironmentPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    target_image: VirtualImage,
    velocity_image: VirtualImage,
    depth: VirtualImage,
    scene_buffer: VirtualBuffer,
}

impl EnvironmentPass {
    pub fn create(
        resources: &PassResources,
        color_format: Format,
        target_image: VirtualImage,
        velocity_format: Format,
        velocity_image: VirtualImage,
        depth: VirtualImage,
        scene_buffer: VirtualBuffer,
    ) -> Result<Self> {
        let pipeline_stages = vec![
            PipelineStageConfig::fragment(shaders::ENVIRONMENT_FRAG),
            PipelineStageConfig::vertex(shaders::ENVIRONMENT_VERT),
        ];

        let pipeline_config = PipelineConfig {
            label: "environment".to_string(),
            stages: pipeline_stages,
            color_formats: vec![color_format, velocity_format],
            depth_format: Some(resources.render_context.depth_format),
            cull_mode: CullModeFlags::NONE,
            depth_write: false,
            ..PipelineConfig::geometry()
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
            velocity_image,
            depth,
            scene_buffer,
        })
    }
}

impl Pass for EnvironmentPass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("environment")
    }

    fn is_enabled(&self, _data_scope: &DataResourceScope) -> bool {
        true
    }

    fn prepare_data(
        &self,
        _data_scope: &mut DataResourceScope,
        _buffer_scope: &mut BufferResourceScope,
        _allocator: &mut HeapAllocator,
        _frame_context: &FrameContext,
    ) -> Result<Self::PassData> {
        Ok(())
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .write_image(
                self.target_image,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_WRITE,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            )
            .write_image(
                self.velocity_image,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_WRITE | AccessFlags::COLOR_ATTACHMENT_READ,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            )
            .read_buffer(
                self.scene_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_image(
                self.depth,
                ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
                PipelineStageFlags::EARLY_FRAGMENT_TESTS | PipelineStageFlags::LATE_FRAGMENT_TESTS,
            );
    }

    fn render_targets(&self) -> Option<RenderTargets> {
        Some(RenderTargets {
            color: vec![
                ColorTarget {
                    image: self.target_image,
                    mip: None,
                    clear: Some(ClearColor::Float([0.0, 0.0, 0.0, 1.0])),
                },
                ColorTarget {
                    image: self.velocity_image,
                    mip: None,
                    clear: None,
                },
            ],
            depth: Some(DepthTarget {
                image: self.depth,
                clear: None,
            }),
            view_mask: 0,
        })
    }

    fn record_commands(
        &self,
        context: &FrameContext,
        _image_scope: &ImageResourceScope,
        buffer_scope: &BufferResourceScope, 
        _readback_scope: &ReadbackScope,
        _data: Self::PassData,
    ) -> Result<()> {
        let scene_buffer = buffer_scope.get_physical_buffer(self.scene_buffer);

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &EnvironmentPushConstants::create(
                scene_buffer.range,
            ),
        );

        context.draw(3);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("EnvironmentRenderPass destroyed");

        Ok(())
    }
}
