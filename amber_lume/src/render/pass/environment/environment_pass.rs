use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use crate::render::pass::pass_resources::PassResources;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, CullModeFlags, Format, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::sync::Arc;
use tracing::info;
use gpu::ResourceFactories;
use crate::render::pass::environment::environment_push_constants::EnvironmentPushConstants;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_image::render_targets::{ClearColor, ColorTarget, DepthTarget, RenderTargets};
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::resources::store::providers::res_ref::ResRef;
use gpu::PipelineLayoutType;
use crate::resources::store::providers::pipeline::pipeline_config::{PipelineConfig, PipelineStageConfig};
use crate::resources::resource_manifest::shaders;

pub struct EnvironmentPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    target_image: VirtualImage,
    depth: VirtualImage,
}

impl EnvironmentPass {
    pub fn create(
        resources: &PassResources,
        color_format: Format,
        target_image: VirtualImage,
        depth: VirtualImage,
    ) -> Result<Self> {
        let pipeline_stages = vec![
            PipelineStageConfig::fragment(shaders::ENVIRONMENT_FRAG),
            PipelineStageConfig::vertex(shaders::ENVIRONMENT_VERT),
        ];

        let pipeline_config = PipelineConfig {
            label: "environment".to_string(),
            stages: pipeline_stages,
            color_formats: vec![color_format],
            depth_format: Some(resources.render_context.depth_format),
            cull_mode: CullModeFlags::NONE,
            depth_write: false,
            ..PipelineConfig::geometry()
        };

        let _handle = resources.pipeline_provider.acquire_sync(pipeline_config);
        let Some(pipeline) = resources.pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire Pipeline");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            target_image,
            depth,
        })
    }
}

pub struct EnvironmentRenderPassData {
    sun_direction: [f32; 3],
    time: f32,
}

impl Pass for EnvironmentPass {
    type PassData = EnvironmentRenderPassData;

    fn name(&self) -> String {
        String::from("environment")
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn prepare_data(
        &self,
        context: &FrameDataContext,
        _buffer_scope: &mut BufferResourceScope,
        _allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        Ok(EnvironmentRenderPassData {
            sun_direction: (-context.render_snapshot.global_shadows_direction).to_array(),
            time: context.render_snapshot.time,
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .write_image(
                self.target_image,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_WRITE,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
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
            color: vec![ColorTarget {
                image: self.target_image,
                mip: None,
                clear: Some(ClearColor::Float([0.0, 0.0, 0.0, 1.0])),
            }],
            depth: Some(DepthTarget {
                image: self.depth,
                clear: None,
            }),
            view_mask: 0,
        })
    }

    fn record_commands(&self, context: &PassContext, _image_scope: &ImageResourceScope, _buffer_scope: &BufferResourceScope, data: Self::PassData) -> Result<()> {
        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &EnvironmentPushConstants::create(
                &context.render_views_layout.main.view_projection,
                data.sun_direction,
                data.time,
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
