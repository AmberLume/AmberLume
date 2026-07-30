use std::sync::atomic::{AtomicBool, Ordering};
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, BlendFactor, BlendOp, ColorComponentFlags, CompareOp, CullModeFlags, Format, FrontFace, ImageLayout, Pipeline, PipelineBindPoint, PipelineStageFlags, PolygonMode, PrimitiveTopology, SampleCountFlags, ShaderStageFlags};
use std::sync::Arc;
use tracing::info;
use gpu::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_graph::pass::Pass;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_image::render_targets::{ColorTarget, RenderTargets};
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use gpu::PipelineLayoutRegistry;
use crate::resources::resource_manifest::shaders;
use crate::resources::store::providers::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::store::providers::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::store::providers::resource_provider::ResourceProvider;

pub struct BrdfLutPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,

    brdf_lut_image: VirtualImage,

    baked: AtomicBool,
}

impl BrdfLutPass {
    pub fn create(
        color_format: Format,
        pipeline_provider: &ResourceProvider<PipelineBackend>,
        _pipeline_layout_registry: &PipelineLayoutRegistry,
        brdf_lut_image: VirtualImage,
    ) -> Result<Self> {
        let pipeline_config = PipelineConfig {
            label: "brdf_lut".to_string(),

            stages: vec![
                PipelineStageConfig {
                    shader_name: shaders::BRDF_LUT_FRAG,
                    fn_name: String::from("main"),
                    stage: ShaderStageFlags::FRAGMENT,
                },
                PipelineStageConfig {
                    shader_name: shaders::FULLSCREEN_VERT,
                    fn_name: String::from("main"),
                    stage: ShaderStageFlags::VERTEX,
                },
            ],

            color_formats: vec![color_format],
            depth_format: None,
            view_mask: 0,

            cull_mode: CullModeFlags::NONE,
            polygon_mode: PolygonMode::FILL,
            front_face: FrontFace::COUNTER_CLOCKWISE,
            primitive_topology: PrimitiveTopology::TRIANGLE_LIST,

            depth_bias_enable: false,
            depth_bias_constant_factor: 0.0,
            depth_bias_slope_factor: 0.0,

            depth_test: false,
            depth_write: false,
            depth_compare_op: CompareOp::ALWAYS,

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

            brdf_lut_image,

            baked: AtomicBool::new(false),
        })
    }
}

impl Pass for BrdfLutPass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("brdf_lut")
    }

    fn is_enabled(&self, _context: &FrameDataContext) -> bool {
        !self.baked.load(Ordering::Relaxed)
    }

    fn prepare_data(
        &self,
        _context: &FrameDataContext,
        _buffer_scope: &mut BufferResourceScope,
        _allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        Ok(())
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .write_image(
                self.brdf_lut_image,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_WRITE,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            );
    }

    fn render_targets(&self) -> Option<RenderTargets> {
        Some(RenderTargets {
            color: vec![ColorTarget {
                image: self.brdf_lut_image,
                mip: None,
                clear: None,
            }],
            depth: None,
            view_mask: 0,
        })
    }

    fn record_commands(&self, context: &PassContext, _image_scope: &ImageResourceScope, _buffer_scope: &BufferResourceScope, _data: Self::PassData) -> Result<()> {
        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.draw(3);

        self.baked.store(true, Ordering::Relaxed);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("BrdfLutPass destroyed");

        Ok(())
    }
}
