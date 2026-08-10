use std::sync::Arc;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, Format, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use tracing::info;
use settings::RenderSettings;
use gpu::ResourceFactories;
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
use gpu::PipelineLayoutType;
use crate::resource_manifest::shaders;
use pipeline_store::PipelineConfig;
use pipeline_store::PipelineStageConfig;
use resource_residency::ResRef;

const DEBUG_LAYER_VELOCITY: usize = 1;
const DEBUG_LAYER_NORMAL: usize = 2;
const DEBUG_LAYER_GTAO: usize = 3;
const DEBUG_LAYER_SH_IRRADIANCE: usize = 4;
const DEBUG_LAYER_HIZ_MIN: usize = 5;
const DEBUG_LAYER_HIZ_MAX: usize = 6;
const DEBUG_LAYER_SHADOW: usize = 7;

pub struct DebugLayerPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    velocity_image: VirtualImage,
    normal_image: VirtualImage,
    gtao_image: VirtualImage,
    sh_image: VirtualImage,
    hiz_image: VirtualImage,
    shadow_history_a: VirtualImage,
    shadow_history_b: VirtualImage,
    shadow_colored: bool,
    target_image: VirtualImage,

}

impl DebugLayerPass {
    pub fn create(
        resources: &PassResources,
        color_format: Format,
        velocity_image: VirtualImage,
        normal_image: VirtualImage,
        gtao_image: VirtualImage,
        sh_image: VirtualImage,
        hiz_image: VirtualImage,
        shadow_history_a: VirtualImage,
        shadow_history_b: VirtualImage,
        shadow_colored: bool,
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
            hiz_image,
            shadow_history_a,
            shadow_history_b,
            shadow_colored,
            target_image,

        })
    }

    fn selected_layer(&self, settings: &RenderSettings) -> usize {
        settings.debug_layer.value
    }
}

impl Pass for DebugLayerPass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("debug_layer")
    }

    fn is_enabled(&self, context: &FrameDataContext) -> bool {
        self.selected_layer(&context.render_settings) != 0
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
            .read_image(
                self.hiz_image,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_image(
                self.shadow_history_a,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_image(
                self.shadow_history_b,
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
        let layer = self.selected_layer(&context.render_settings);

        let source = match layer {
            DEBUG_LAYER_VELOCITY => self.velocity_image,
            DEBUG_LAYER_NORMAL => self.normal_image,
            DEBUG_LAYER_GTAO => self.gtao_image,
            DEBUG_LAYER_SH_IRRADIANCE => self.sh_image,
            DEBUG_LAYER_HIZ_MIN | DEBUG_LAYER_HIZ_MAX => self.hiz_image,
            DEBUG_LAYER_SHADOW => if context.history_write_index == 0 {
                self.shadow_history_a
            } else {
                self.shadow_history_b
            },
            _ => unreachable!(),
        };

        let source = image_scope.get_physical_image(source);

        let texture_index = if layer == DEBUG_LAYER_HIZ_MIN || layer == DEBUG_LAYER_HIZ_MAX {
            let Some(mips) = source.descriptors.sampled_mips.as_ref().filter(|mips| !mips.is_empty()) else {
                return Ok(());
            };
            let requested = context.render_settings.hiz_mip.value as usize;
            mips[requested.min(mips.len() - 1)]
        } else {
            let Some(index) = source.descriptors.full else {
                return Ok(());
            };
            index
        };

        let inverse_view_projection = context.render_views_layout.main.view_projection
            .inverted().value.to_cols_array_2d();

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &DebugLayerPushConstants::create(
                inverse_view_projection,
                texture_index.inner,
                layer as u32,
                self.shadow_colored as u32,
            ),
        );

        context.draw(3);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("DebugLayerPass destroyed");

        Ok(())
    }
}
