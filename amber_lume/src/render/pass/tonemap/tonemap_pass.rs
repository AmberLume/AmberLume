use std::sync::Arc;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, Format, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use tracing::info;
use gpu::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_context::PassContext;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::tonemap::tonemap_push_constants::TonemapPushConstants;
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

        let _handle = resources.pipeline_provider.acquire_sync(pipeline_config);
        let Some(pipeline) = resources.pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire Pipeline");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            scene_color,
            history_a,
            history_b,
            bloom_image,
            target_image,

            hdr,
        })
    }
}

impl Pass for TonemapPass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("tonemap")
    }

    fn is_enabled(&self, _context: &FrameDataContext) -> bool {
        true
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

    fn record_commands(&self, context: &PassContext, image_scope: &ImageResourceScope, _buffer_scope: &BufferResourceScope, _data: Self::PassData) -> Result<()> {
        let input_image = if context.render_settings.fsr_enabled.value {
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

        let settings = context.render_settings;
        context.push_constants(
            self.pipeline_layout,
            &TonemapPushConstants::create(
                input_texture.inner,
                settings.exposure.value,
                self.hdr as u32,
                settings.paper_white.value,
                bloom_texture.inner,
                settings.bloom_intensity.value,
                settings.sharpness.value,
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
