use std::sync::Arc;
use anyhow::{bail, Result};
use arc_swap::ArcSwap;
use ash::vk::{AccessFlags, BlendFactor, BlendOp, ColorComponentFlags, CompareOp, CullModeFlags, Format, FrontFace, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PolygonMode, PrimitiveTopology, SampleCountFlags, ShaderStageFlags};
use tracing::info;
use crate::settings::settings::EngineSettings;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::debug_layer::debug_layer_push_constants::DebugLayerPushConstants;
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

pub struct DebugLayerPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    velocity_image: VirtualImage,
    normal_image: VirtualImage,
    gtao_image: VirtualImage,
    target_image: VirtualImage,

    settings: Arc<ArcSwap<EngineSettings>>,
}

impl DebugLayerPass {
    pub fn create(
        color_format: Format,
        pipeline_provider: &ResourceProvider<PipelineBackend>,
        pipeline_layout_registry: &PipelineLayoutRegistry,
        velocity_image: VirtualImage,
        normal_image: VirtualImage,
        gtao_image: VirtualImage,
        target_image: VirtualImage,
        settings: Arc<ArcSwap<EngineSettings>>,
    ) -> Result<Self> {
        let pipeline_config = PipelineConfig {
            label: "debug_layer".to_string(),

            stages: vec![
                PipelineStageConfig {
                    shader_name: shaders::DEBUG_LAYER_FRAG,
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
            pipeline_layout: pipeline_layout_registry.get(PipelineLayoutType::General),

            velocity_image,
            normal_image,
            gtao_image,
            target_image,

            settings,
        })
    }

    fn selected_layer(&self) -> usize {
        self.settings.load().debug.debug_layer.value
    }
}

impl Pass for DebugLayerPass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("debug_layer")
    }

    fn is_enabled(&self) -> bool {
        self.selected_layer() != 0
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
            .read_image(
                self.velocity_image,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_image(
                self.normal_image,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_image(
                self.gtao_image,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            )
            .write_image(
                self.target_image,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_WRITE,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            );
    }

    fn render_targets(&self) -> Option<RenderTargets> {
        Some(RenderTargets {
            color: vec![ColorTarget {
                image: self.target_image,
                clear: None,
            }],
            depth: None,
            view_mask: 0,
        })
    }

    fn record_commands(&self, context: &PassContext, image_scope: &ImageResourceScope, _buffer_scope: &BufferResourceScope, _data: Self::PassData) -> Result<()> {
        let layer = self.selected_layer();

        let source = match layer {
            1 => self.velocity_image,
            2 => self.normal_image,
            3 => self.gtao_image,
            _ => unreachable!(),
        };

        let source = image_scope.get_physical_image(source);
        let Some(texture_index) = source.descriptors.full else {
            return Ok(());
        };

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &DebugLayerPushConstants {
                texture_index,
                layer_kind: layer as u32,
            },
        );

        context.draw(3);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("DebugLayerPass destroyed");

        Ok(())
    }
}
