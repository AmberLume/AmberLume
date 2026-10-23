use render_graph::VirtualData;
use std::sync::Arc;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, Format, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use tracing::info;
use gpu::ResourceFactories;
use crate::render::pass::bloom::bloom_downsample_push_constants::BloomDownsamplePushConstants;
use crate::resource_manifest::shaders;
use render_graph::FrameContext;
use crate::render::pass_resources::pass_resources::PassResources;
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
use render_graph::PrepareScopes;
use render_graph::RecordScopes;
use render_graph::DataResourceScope;
use render_graph::{ColorTarget, RenderTargets};
use render_graph::VirtualImage;
use gpu::PipelineLayoutType;
use pipeline_store::PipelineConfig;
use pipeline_store::PipelineStageConfig;
use resource_residency::ResRef;
use settings::RenderSettings;

pub struct BloomDownsamplePass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    render_settings: VirtualData<RenderSettings>,

    src: VirtualImage,
    dst: VirtualImage,

    src_mip: Option<u32>,
    dst_mip: u32,
    karis_average: bool,
}

impl BloomDownsamplePass {
    pub fn create(
        resources: &PassResources,
        color_format: Format,
        src: VirtualImage,
        src_mip: Option<u32>,
        dst: VirtualImage,
        dst_mip: u32,
        karis_average: bool,
        render_settings: VirtualData<RenderSettings>,
    ) -> Result<Self> {
        let pipeline_config = PipelineConfig {
            label: "bloom_downsample".to_string(),

            stages: vec![
                PipelineStageConfig::vertex(shaders::BLOOM_DOWNSAMPLE),
                PipelineStageConfig::fragment(shaders::BLOOM_DOWNSAMPLE),
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

            render_settings,

            src,
            dst,

            src_mip,
            dst_mip,
            karis_average,
        })
    }
}

pub struct BloomDownsamplePassData {
    threshold: f32,
}

impl Pass for BloomDownsamplePass {
    type PassData = BloomDownsamplePassData;

    fn name(&self) -> String {
        format!("bloom_downsample_{}", self.dst_mip)
    }

    fn is_enabled(&self, data_scope: &DataResourceScope) -> bool {
        data_scope.get(self.render_settings).bloom_intensity.value > 0.0
    }

    fn prepare_data(
        &self,
        scopes: &mut PrepareScopes,
        _frame_context: &FrameContext,
    ) -> Result<Self::PassData> {
        let render_settings = scopes.data.get(self.render_settings);

        Ok(BloomDownsamplePassData {
            threshold: render_settings.bloom_threshold.value,
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration.consume(self.render_settings);

        match self.src_mip {
            Some(mip) => declaration.read_image_mip(
                self.src,
                mip,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            ),
            None => declaration.read_image(
                self.src,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            ),
        };

        declaration.write_image_mip(
            self.dst,
            self.dst_mip,
            ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            AccessFlags::COLOR_ATTACHMENT_WRITE,
            PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        );
    }

    fn render_targets(&self) -> Option<RenderTargets> {
        Some(RenderTargets {
            color: vec![ColorTarget {
                image: self.dst,
                mip: Some(self.dst_mip),
                clear: None,
            }],
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
        let src = scopes.image.get_physical_image(self.src);
        let source_descriptor_id = match self.src_mip {
            Some(mip) => src.descriptors.sampled_mips.as_ref().and_then(|slots| slots.get(mip as usize).copied()),
            None => src.descriptors.full,
        };
        let Some(source_descriptor_id) = source_descriptor_id else {
            return Ok(());
        };

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &BloomDownsamplePushConstants::create(
                source_descriptor_id.inner,
                self.karis_average as u32,
                data.threshold,
            ),
        );

        context.draw(3);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("BloomDownsamplePass destroyed");

        Ok(())
    }
}
