use render_graph::ReadbackScope;
use render_graph::VirtualData;
use render_snapshot::RenderSnapshot;
use crate::render::pass::selection_mask::selection_mask_pass::SelectionMaskPass;
use std::sync::Arc;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, Format, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use tracing::info;
use gpu::ResourceFactories;
use render_graph::FrameContext;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::selection::selection_push_constants::SelectionPushConstants;
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
use render_graph::ImageResourceScope;
use render_graph::BufferResourceScope;
use render_graph::DataResourceScope;
use render_graph::HeapAllocator;
use render_graph::{ColorTarget, RenderTargets};
use render_graph::VirtualImage;
use render_graph::VirtualBuffer;
use gpu::PipelineLayoutType;
use crate::resource_manifest::shaders;
use pipeline_store::BlendConfig;
use pipeline_store::PipelineConfig;
use pipeline_store::PipelineStageConfig;
use resource_residency::ResRef;

pub struct SelectionPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    target_image: VirtualImage,
    entity_id_image: VirtualImage,
    mask_image: VirtualImage,
    entity_buffer: VirtualBuffer,
    scene_buffer: VirtualBuffer,

    render_snapshot: VirtualData<RenderSnapshot>,
}

impl SelectionPass {
    pub const OUTLINE_RADIUS: i32 = 5;

    pub fn create(
        resources: &PassResources,
        color_format: Format,
        target_image: VirtualImage,
        entity_id_image: VirtualImage,
        mask_image: VirtualImage,
        entity_buffer: VirtualBuffer,
        scene_buffer: VirtualBuffer,
        render_snapshot: VirtualData<RenderSnapshot>,
    ) -> Result<Self> {
        let pipeline_config = PipelineConfig {
            label: "selection".to_string(),

            stages: vec![
                PipelineStageConfig::fragment(shaders::SELECTION_FRAG),
                PipelineStageConfig::vertex(shaders::SELECTION_VERT),
            ],

            color_formats: vec![color_format],

            blend_enabled: true,
            color_blend: Some(BlendConfig::alpha()),
            alpha_blend: Some(BlendConfig::replace()),

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

            target_image,
            entity_id_image,
            mask_image,
            entity_buffer,
            scene_buffer,

            render_snapshot,
        })
    }
}

impl Pass for SelectionPass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("selection")
    }

    fn is_enabled(&self, data_scope: &DataResourceScope) -> bool {
        let render_snapshot = data_scope.get(self.render_snapshot);

        render_snapshot.entities.iter().any(|entity| entity.outline[3] > 0.0)
    }

    fn prepare_data(
        &self,
        _data_scope: &mut DataResourceScope,
        _buffer_scope: &mut BufferResourceScope,
        _allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        Ok(())
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .consume(self.render_snapshot)
            .read_image(
                self.mask_image,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_image(
                self.entity_id_image,
                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_buffer(
                self.entity_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            )
            .read_buffer(
                self.scene_buffer,
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
        buffer_scope: &BufferResourceScope,
        _readback_scope: &ReadbackScope,
        _data: Self::PassData,
    ) -> Result<()> {
        let entity_id_image = image_scope.get_physical_image(self.entity_id_image);
        let entity_buffer = buffer_scope.get_physical_buffer(self.entity_buffer);
        let scene_buffer = buffer_scope.get_physical_buffer(self.scene_buffer);

        let mask_image = image_scope.get_physical_image(self.mask_image);

        let (Some(entity_id_texture), Some(mask_texture)) =
            (entity_id_image.descriptors.full, mask_image.descriptors.full) else {
            return Ok(());
        };

        let target = image_scope.get_physical_image(self.target_image);
        let entity_id_texel_scale = [
            entity_id_image.extent.width as f32 / target.extent.width as f32,
            entity_id_image.extent.height as f32 / target.extent.height as f32,
        ];

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &SelectionPushConstants::create(
                scene_buffer.range,
                entity_buffer.range,
                entity_id_texel_scale,
                entity_id_texture.inner,
                mask_texture.inner,
                Self::OUTLINE_RADIUS,
                SelectionMaskPass::MASK_SCALE,
            ),
        );

        context.draw(3);

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("SelectionPass destroyed");

        Ok(())
    }
}
