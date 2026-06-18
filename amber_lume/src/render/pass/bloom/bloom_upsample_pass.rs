use std::sync::Arc;
use anyhow::{bail, Result};
use arc_swap::ArcSwap;
use ash::vk::{AccessFlags, Format, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use tracing::info;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::pass::bloom::bloom_push_constants::BloomPushConstants;
use crate::settings::settings::EngineSettings;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_context::PassContext;
use crate::render::pass::pass_resources::PassResources;
use crate::render::render_graph::pass::Pass;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_image::render_targets::{ColorTarget, RenderTargets};
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::resources::binding_layout::pipeline_layout_registry::PipelineLayoutType;
use crate::resources::resource_manifest::shaders;
use crate::resources::store::providers::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};
use crate::resources::store::providers::res_ref::ResRef;

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
        resources: &PassResources,
        color_format: Format,
        image: VirtualImage,
        src_mip: u32,
        dst_mip: u32,
    ) -> Result<Self> {
        let pipeline_config = PipelineConfig {
            label: "bloom_upsample".to_string(),

            stages: vec![
                PipelineStageConfig::fragment(shaders::UPSAMPLE_FRAG),
                PipelineStageConfig::vertex(shaders::FULLSCREEN_VERT),
            ],

            color_formats: vec![color_format],

            blend_enabled: true,
            color_blend: Some(BlendConfig::additive()),

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

            image,
            src_mip,
            dst_mip,

            settings: resources.settings.clone(),
        })
    }
}

impl Pass for BloomUpsamplePass {
    type PassData = ();

    fn name(&self) -> String {
        format!("bloom_upsample_{}", self.dst_mip)
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
            &BloomPushConstants::create(src_texture, 0, 0.0),
        );

        context.draw(3);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("BloomUpsamplePass destroyed");

        Ok(())
    }
}
