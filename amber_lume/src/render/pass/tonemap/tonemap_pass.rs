use render_graph::ReadbackScope;
use render_graph::VirtualData;
use settings::RenderSettings;
use std::sync::Arc;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, Format, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use tracing::info;
use gpu::ResourceFactories;
use render_graph::FrameContext;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::tonemap::tonemap_push_constants::TonemapPushConstants;
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
use render_graph::ImageResourceScope;
use render_graph::BufferResourceScope;
use render_graph::DataResourceScope;
use render_graph::{ColorTarget, RenderTargets};
use render_graph::VirtualImage;
use gpu::PipelineLayoutType;
use crate::resource_manifest::shaders;
use pipeline_store::PipelineConfig;
use pipeline_store::PipelineStageConfig;
use resource_residency::ResRef;

pub struct TonemapPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    scene_color: VirtualImage,
    history_a: VirtualImage,
    history_b: VirtualImage,
    bloom_image: VirtualImage,
    target_image: VirtualImage,

    hdr: bool,

    render_settings: VirtualData<RenderSettings>,
}

impl TonemapPass {
    pub fn create(
        resources: &PassResources,
        color_format: Format,
        scene_color: VirtualImage,
        history_a: VirtualImage,
        history_b: VirtualImage,
        bloom_image: VirtualImage,
        target_image: VirtualImage,
        hdr: bool,
        render_settings: VirtualData<RenderSettings>,
    ) -> Result<Self> {
        let pipeline_config = PipelineConfig {
            label: "tonemap".to_string(),
            stages: vec![
                PipelineStageConfig::fragment(shaders::TONEMAP_FRAG),
                PipelineStageConfig::vertex(shaders::TONEMAP_VERT),
            ],
            color_formats: vec![color_format],
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

            scene_color,
            history_a,
            history_b,
            bloom_image,
            target_image,

            hdr,
        
            render_settings,
        })
    }
}

pub struct TonemapPassData {
    fsr_enabled: bool,

    exposure: f32,
    paper_white: f32,
    bloom_intensity: f32,
    sharpness: f32,
}

impl Pass for TonemapPass {
    type PassData = TonemapPassData;

    fn name(&self) -> String {
        String::from("tonemap")
    }

    fn is_enabled(&self, _data_scope: &DataResourceScope) -> bool {
        true
    }

    fn prepare_data(
        &self,
        data_scope: &mut DataResourceScope,
        _buffer_scope: &mut BufferResourceScope,
        _frame_context: &FrameContext,
    ) -> Result<Self::PassData> {
        let settings = data_scope.get(self.render_settings);

        Ok(TonemapPassData {
            fsr_enabled: settings.fsr_enabled.value,

            exposure: settings.exposure.value,
            paper_white: settings.paper_white.value,
            bloom_intensity: settings.bloom_intensity.value,
            sharpness: settings.sharpness.value,
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .consume(self.render_settings)
            .read_image(
                self.scene_color,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_image(
                self.history_a,
                ImageLayout::GENERAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_image(
                self.history_b,
                ImageLayout::GENERAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_image_mip(
                self.bloom_image,
                0,
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

    fn record_commands(
        &self,
        context: &FrameContext,
        image_scope: &ImageResourceScope,
        _buffer_scope: &BufferResourceScope,
        _readback_scope: &ReadbackScope,
        data: Self::PassData,
    ) -> Result<()> {
        let input_image = if data.fsr_enabled {
            if context.history_write_index == 0 {
                self.history_a
            } else {
                self.history_b
            }
        } else {
            self.scene_color
        };

        let input = image_scope.get_physical_image(input_image);
        let Some(input_texture) = input.descriptors.full else {
            return Ok(());
        };

        let bloom = image_scope.get_physical_image(self.bloom_image);
        let bloom_texture = bloom.descriptors.sampled_mips.as_ref()
            .and_then(|slots| slots.get(0).copied());
        let Some(bloom_texture) = bloom_texture else {
            return Ok(());
        };

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &TonemapPushConstants::create(
                input_texture.inner,
                data.exposure,
                self.hdr as u32,
                data.paper_white,
                bloom_texture.inner,
                data.bloom_intensity,
                data.sharpness,
            ),
        );

        context.draw(3);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("TonemapPass destroyed");

        Ok(())
    }
}
