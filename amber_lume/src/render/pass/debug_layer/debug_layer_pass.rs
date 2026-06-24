use std::sync::Arc;
use anyhow::{bail, Result};
use arc_swap::ArcSwap;
use ash::vk::{AccessFlags, Format, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use tracing::info;
use crate::settings::settings::EngineSettings;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::debug_layer::debug_layer_push_constants::DebugLayerPushConstants;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::pass_context::PassContext;
use crate::render::render_graph::pass::Pass;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_image::render_targets::{ColorTarget, RenderTargets};
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::resources::binding_layout::pipeline_layout_registry::PipelineLayoutType;
use crate::resources::resource_manifest::shaders;
use crate::resources::store::providers::pipeline::pipeline_config::{PipelineConfig, PipelineStageConfig};
use crate::resources::store::providers::res_ref::ResRef;

const DEBUG_LAYER_VELOCITY: usize = 1;
const DEBUG_LAYER_NORMAL: usize = 2;
const DEBUG_LAYER_GTAO: usize = 3;
const DEBUG_LAYER_SH_IRRADIANCE: usize = 4;

pub struct DebugLayerPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    velocity_image: VirtualImage,
    normal_image: VirtualImage,
    gtao_image: VirtualImage,
    sh_image: VirtualImage,
    target_image: VirtualImage,

    settings: Arc<ArcSwap<EngineSettings>>,
}

impl DebugLayerPass {
    pub fn create(
        resources: &PassResources,
        color_format: Format,
        velocity_image: VirtualImage,
        normal_image: VirtualImage,
        gtao_image: VirtualImage,
        sh_image: VirtualImage,
        target_image: VirtualImage,
    ) -> Result<Self> {
        let pipeline_config = PipelineConfig {
            label: "debug_layer".to_string(),

            stages: vec![
                PipelineStageConfig::fragment(shaders::DEBUG_LAYER_FRAG),
                PipelineStageConfig::vertex(shaders::FULLSCREEN_VERT),
            ],

            color_formats: vec![color_format],

            ..PipelineConfig::fullscreen()
        };

        let _handle = resources.pipeline_provider.acquire_sync(pipeline_config);
        let Some(pipeline) = resources.pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire Pipeline");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            velocity_image,
            normal_image,
            gtao_image,
            sh_image,
            target_image,

            settings: resources.settings.clone(),
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
            .read_image(
                self.sh_image,
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
                mip: None,
                clear: None,
            }],
            depth: None,
            view_mask: 0,
        })
    }

    fn record_commands(&self, context: &PassContext, image_scope: &ImageResourceScope, _buffer_scope: &BufferResourceScope, _data: Self::PassData) -> Result<()> {
        let layer = self.selected_layer();

        let source = match layer {
            DEBUG_LAYER_VELOCITY => self.velocity_image,
            DEBUG_LAYER_NORMAL => self.normal_image,
            DEBUG_LAYER_GTAO => self.gtao_image,
            DEBUG_LAYER_SH_IRRADIANCE => self.sh_image,
            _ => unreachable!(),
        };

        let source = image_scope.get_physical_image(source);
        let Some(texture_index) = source.descriptors.full else {
            return Ok(());
        };

        let inverse_view_projection = context.render_views_layout.main.view_projection
            .inverted().value.to_cols_array_2d();

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &DebugLayerPushConstants::create(inverse_view_projection, texture_index, layer as u32),
        );

        context.draw(3);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("DebugLayerPass destroyed");

        Ok(())
    }
}
