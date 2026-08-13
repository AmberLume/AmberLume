use render_graph::ReadbackScope;
use render_graph::VirtualData;
use std::sync::Arc;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, Format, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use tracing::info;
use gpu::ResourceFactories;
use crate::render::pass::bloom::bloom_push_constants::BloomPushConstants;
use render_graph::FrameContext;
use crate::render::pass::pass_resources::PassResources;
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
use render_graph::ImageResourceScope;
use render_graph::BufferResourceScope;
use render_graph::DataResourceScope;
use render_graph::HeapAllocator;
use render_graph::{ColorTarget, RenderTargets};
use render_graph::VirtualImage;
use gpu::PipelineLayoutType;
use crate::resource_manifest::shaders;
use pipeline_store::PipelineConfig;
use pipeline_store::PipelineStageConfig;
use resource_residency::ResRef;
use settings::RenderSettings;

pub struct BloomDownsamplePass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    src: VirtualImage,
    src_mip: Option<u32>,
    dst: VirtualImage,
    dst_mip: u32,

    karis: bool,

    render_settings: VirtualData<RenderSettings>,
}

impl BloomDownsamplePass {
    pub fn create(
        resources: &PassResources,
        color_format: Format,
        src: VirtualImage,
        src_mip: Option<u32>,
        dst: VirtualImage,
        dst_mip: u32,
        karis: bool,
        render_settings: VirtualData<RenderSettings>,
    ) -> Result<Self> {
        let pipeline_config = PipelineConfig {
            label: "bloom_downsample".to_string(),

            stages: vec![
                PipelineStageConfig::fragment(shaders::DOWNSAMPLE_FRAG),
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

            src,
            src_mip,
            dst,
            dst_mip,

            karis,

            render_settings,
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
        data_scope: &mut DataResourceScope,
        _buffer_scope: &mut BufferResourceScope,
        _allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        let render_settings = data_scope.get(self.render_settings);

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
        image_scope: &ImageResourceScope,
        _buffer_scope: &BufferResourceScope,
        _readback_scope: &ReadbackScope,
        data: Self::PassData,
    ) -> Result<()> {
        let src = image_scope.get_physical_image(self.src);
        let src_texture = match self.src_mip {
            Some(mip) => src.descriptors.sampled_mips.as_ref().and_then(|slots| slots.get(mip as usize).copied()),
            None => src.descriptors.full,
        };
        let Some(src_texture) = src_texture else {
            return Ok(());
        };

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &BloomPushConstants::create(src_texture.inner, self.karis as u32, data.threshold),
        );

        context.draw(3);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("BloomDownsamplePass destroyed");

        Ok(())
    }
}
