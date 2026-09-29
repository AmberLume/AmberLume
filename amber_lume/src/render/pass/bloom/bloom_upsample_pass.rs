use render_graph::VirtualData;
use settings::RenderSettings;
use std::sync::Arc;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, Format, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use tracing::info;
use gpu::ResourceFactories;
use crate::render::pass::bloom::upsample_fragment_shader::UpsampleFragmentShader;
use crate::render::push_constants::pipeline_push_constants::PipelinePushConstants;
use crate::render::push_constants::push_constants::PushConstants;
use crate::render::push_constants::vertex::fullscreen_uv_vertex_shader::FullscreenUvVertexShader;
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
use pipeline_store::BlendConfig;
use pipeline_store::PipelineConfig;
use resource_residency::ResRef;

pub struct BloomUpsamplePass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,
    push_constants: PipelinePushConstants<FullscreenUvVertexShader, UpsampleFragmentShader>,

    render_settings: VirtualData<RenderSettings>,

    image: VirtualImage,

    src_mip: u32,
    dst_mip: u32,
}

impl BloomUpsamplePass {
    pub fn create(
        resources: &PassResources,
        color_format: Format,
        image: VirtualImage,
        src_mip: u32,
        dst_mip: u32,
        render_settings: VirtualData<RenderSettings>,
    ) -> Result<Self> {
        let push_constants = PipelinePushConstants::<FullscreenUvVertexShader, UpsampleFragmentShader>::new();

        let pipeline_config = PipelineConfig {
            label: "bloom_upsample".to_string(),

            stages: push_constants.stages_layout(),

            color_formats: vec![color_format],

            blend_enabled: true,
            color_blend: Some(BlendConfig::additive()),

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
            push_constants,

            render_settings,

            image,

            src_mip,
            dst_mip,
        })
    }
}

impl Pass for BloomUpsamplePass {
    type PassData = ();

    fn name(&self) -> String {
        format!("bloom_upsample_{}", self.dst_mip)
    }

    fn is_enabled(&self, data_scope: &DataResourceScope) -> bool {
        data_scope.get(self.render_settings).bloom_intensity.value > 0.0
    }

    fn prepare_data(
        &self,
        _scopes: &mut PrepareScopes,
        _frame_context: &FrameContext,
    ) -> Result<Self::PassData> {
        Ok(())
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .consume(self.render_settings)
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

    fn record_commands(
        &self,
        context: &FrameContext,
        scopes: &RecordScopes,
        _data: Self::PassData,
    ) -> Result<()> {
        let bloom = scopes.image.get_physical_image(self.image);
        let src_texture = bloom.descriptors.sampled_mips.as_ref()
            .and_then(|slots| slots.get(self.src_mip as usize).copied());
        let Some(src_texture) = src_texture else {
            return Ok(());
        };

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        self.push_constants.push(
            context,
            self.pipeline_layout,
            &PushConstants {
                vertex: FullscreenUvVertexShader,
                fragment: UpsampleFragmentShader::create(src_texture.inner),
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
