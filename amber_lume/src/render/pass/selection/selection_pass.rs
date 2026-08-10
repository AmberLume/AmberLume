use render_graph::VirtualData;
use settings::RenderSettings;
use std::sync::Arc;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, Format, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use tracing::info;
use gpu::ResourceFactories;
use render_graph::FrameContext;
use crate::render::pass::pass_resources::PassResources;
use crate::render::pass::selection::selection_push_constants::SelectionPushConstants;
use crate::render::readback::entity_id_pick_reader::EntityIdPickReader;
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
use pipeline_store::BlendConfig;
use pipeline_store::PipelineConfig;
use pipeline_store::PipelineStageConfig;
use resource_residency::ResRef;

const STRIPE_WIDTH: f32 = 8.0;

pub struct SelectionPass {
    _handle: Arc<ResRef>,

    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    target_image: VirtualImage,
    entity_id_image: VirtualImage,

    color: [f32; 4],

    pick_reader: Arc<EntityIdPickReader>,

    render_settings: VirtualData<RenderSettings>,
}

impl SelectionPass {
    pub fn create(
        resources: &PassResources,
        color_format: Format,
        target_image: VirtualImage,
        entity_id_image: VirtualImage,
        color: [f32; 4],
        pick_reader: Arc<EntityIdPickReader>,
        render_settings: VirtualData<RenderSettings>,
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

        let _handle = resources.pipeline_provider.acquire_sync(pipeline_config);
        let Some(pipeline) = resources.pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire Pipeline");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            target_image,
            entity_id_image,

            color,

            pick_reader,
        
            render_settings,
        })
    }
}

impl Pass for SelectionPass {
    type PassData = ();

    fn name(&self) -> String {
        String::from("selection")
    }

    fn is_enabled(&self, data_scope: &DataResourceScope) -> bool {
        data_scope.get(self.render_settings).selection_enabled.value
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
            .consume(self.render_settings)
            .read_image(
                self.entity_id_image,
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
        _data: Self::PassData,
    ) -> Result<()> {
        let Some(selected_entity) = self.pick_reader.value() else {
            return Ok(());
        };

        let entity_id = image_scope.get_physical_image(self.entity_id_image);
        let Some(entity_id_texture) = entity_id.descriptors.full else {
            return Ok(());
        };

        let target = image_scope.get_physical_image(self.target_image);
        let entity_id_texel_scale = [
            entity_id.extent.width as f32 / target.extent.width as f32,
            entity_id.extent.height as f32 / target.extent.height as f32,
        ];

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.push_constants(
            self.pipeline_layout,
            &SelectionPushConstants::create(
                self.color,
                entity_id_texel_scale,
                entity_id_texture.inner,
                selected_entity,
                STRIPE_WIDTH,
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
