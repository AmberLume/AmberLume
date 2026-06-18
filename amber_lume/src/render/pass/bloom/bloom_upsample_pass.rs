use std::sync::Arc;
use anyhow::{bail, Result};
use arc_swap::ArcSwap;
use ash::vk::{AccessFlags, BlendFactor, BlendOp, ColorComponentFlags, CompareOp, CullModeFlags, Format, FrontFace, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PolygonMode, PrimitiveTopology, SampleCountFlags, ShaderStageFlags};
use tracing::info;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::bloom::bloom_push_constants::BloomPushConstants;
use crate::settings::settings::EngineSettings;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_graph::pass::Pass;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_image::render_targets::{ColorTarget, RenderTargets};
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::resources::binding_layout::pipeline_layout_registry::{PipelineLayoutRegistry, PipelineLayoutType};
use crate::resources::resource_manifest::shaders;
use crate::resources::store::providers::pipeline::pipeline_backend::PipelineBackend;
use crate::resources::store::providers::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::store::providers::resource_provider::ResourceProvider;

pub struct BloomUpsamplePass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    image: VirtualImage,
    src_mip: u32,
    dst_mip: u32,

    settings: Arc<ArcSwap<EngineSettings>>,
}

impl BloomUpsamplePass {
    pub fn create(
        color_format: Format,
        pipeline_provider: &ResourceProvider<PipelineBackend>,
        pipeline_layout_registry: &PipelineLayoutRegistry,
        image: VirtualImage,
        src_mip: u32,
        dst_mip: u32,
        settings: Arc<ArcSwap<EngineSettings>>,
    ) -> Result<Self> {
        let pipeline_config = PipelineConfig {
            label: "bloom_upsample".to_string(),

            stages: vec![
                PipelineStageConfig {
                    shader_name: shaders::UPSAMPLE_FRAG,
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

            blend_enabled: true,
            color_blend: Some(BlendConfig {
                blend_op: BlendOp::ADD,
                src_blend: BlendFactor::ONE,
                dst_blend: BlendFactor::ONE,
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

            image,
            src_mip,
            dst_mip,

            settings,
        })
    }
}

impl Pass for BloomUpsamplePass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("bloom_upsample")
    }

    fn is_enabled(&self) -> bool {
        self.settings.load().render.bloom_intensity.value > 0.0
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
            .read_image_mip(
                self.image,
                self.src_mip,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            )
            .write_image_mip(
                self.image,
                self.dst_mip,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_WRITE | AccessFlags::COLOR_ATTACHMENT_READ,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            );
    }

    fn render_targets(&self) -> Option<RenderTargets> {
        Some(RenderTargets {
            color: vec![ColorTarget {
                image: self.image,
                mip: Some(self.dst_mip),
                clear: None,
            }],
            depth: None,
            view_mask: 0,
        })
    }

    fn record_commands(&self, context: &PassContext, image_scope: &ImageResourceScope, _buffer_scope: &BufferResourceScope, _data: Self::PassData) -> Result<()> {
        let bloom = image_scope.get_physical_image(self.image);
        let src_texture = bloom.descriptors.sampled_mips.as_ref()
            .and_then(|slots| slots.get(self.src_mip as usize).copied());
        let Some(src_texture) = src_texture else {
            return Ok(());
        };

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &BloomPushConstants {
                src_texture,
                karis: 0,
                threshold: 0.0,
            },
        );

        context.draw(3);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("BloomUpsamplePass destroyed");

        Ok(())
    }
}
