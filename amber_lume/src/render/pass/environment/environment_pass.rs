use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_context::RenderContext;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, BlendFactor, BlendOp, ColorComponentFlags, CompareOp, CullModeFlags, Format, FrontFace, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PolygonMode, PrimitiveTopology, SampleCountFlags, ShaderStageFlags};
use std::sync::Arc;
use tracing::info;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::environment::environment_push_constants::EnvironmentPushConstants;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_image::render_targets::{ClearColor, ColorTarget, DepthTarget, RenderTargets};
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::store::providers::resource_provider::ResourceProvider;
use crate::resources::binding_layout::pipeline_layout_registry::{PipelineLayoutRegistry, PipelineLayoutType};
use crate::resources::store::providers::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::store::providers::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};
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
        color_format: Format,
        render_context: &RenderContext,
        pipeline_provider: &ResourceProvider<PipelineBackend>,
        pipeline_layout_registry: &PipelineLayoutRegistry,
        target_image: VirtualImage,
        depth: VirtualImage,
    ) -> Result<Self> {
        let pipeline_stages = vec![
            PipelineStageConfig {
                shader_name: shaders::ENVIRONMENT_FRAG,
                fn_name: String::from("main"),
                stage: ShaderStageFlags::FRAGMENT,
            },
            PipelineStageConfig {
                shader_name: shaders::ENVIRONMENT_VERT,
                fn_name: String::from("main"),
                stage: ShaderStageFlags::VERTEX,
            },
        ];

        let pipeline_config = PipelineConfig {
            label: "environment".to_string(),

            stages: pipeline_stages,

            color_formats: vec![color_format],
            depth_format: Some(render_context.depth_format),
            view_mask: 0,

            cull_mode: CullModeFlags::NONE,
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

            target_image,
            depth,
        })
    }
}

pub struct EnvironmentRenderPassData {
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
                data.time,
                50.0,
                0.45,
                0.06,
                [0.78, 0.86, 1.0],
                [1.0, 0.9, 0.78],
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
