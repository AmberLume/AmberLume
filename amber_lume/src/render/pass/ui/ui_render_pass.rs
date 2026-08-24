use render_graph::ReadbackScope;
use render_graph::VirtualData;
use render_graph::Pass;
use render_graph::FrameContext;
use crate::render::pass::pass_resources::PassResources;
use anyhow::{bail, Result};
use ash::vk::{Offset2D, Extent2D, AccessFlags, Format, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::sync::Arc;
use tracing::info;
use gpu::ResourceFactories;
use crate::render::pass::ui::ui_push_constants::UiPushConstants;
use ui::UiDrawLayer;
use ui::UiFrame;
use render_graph::PassResourceDeclaration;
use render_graph::ImageResourceScope;
use render_graph::BufferResourceScope;
use render_graph::DataResourceScope;
use render_graph::HeapAllocator;
use render_graph::{ColorTarget, RenderTargets};
use render_graph::VirtualBuffer;
use render_graph::VirtualImage;
use resource_residency::ResRef;
use gpu::PipelineLayoutType;
use pipeline_store::BlendConfig;
use pipeline_store::PipelineConfig;
use pipeline_store::PipelineStageConfig;
use crate::resource_manifest::shaders;
use gpu::BufferRange;

pub struct UiPass {
    _handle: Arc<ResRef>,
    
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    index_buffer: VirtualBuffer,
    vertex_buffer: VirtualBuffer,

    target_image: VirtualImage,

    ui_frame: VirtualData<UiFrame>,
}

impl UiPass {
    pub fn create(
        resources: &PassResources,
        index_buffer: VirtualBuffer,
        vertex_buffer: VirtualBuffer,
        color_format: Format,
        target_image: VirtualImage,
        ui_frame: VirtualData<UiFrame>,
    ) -> Result<Self> {
        let pipeline_config = PipelineConfig {
            label: "ui".to_string(),
            stages: vec![
                PipelineStageConfig::fragment(shaders::YAKUI_FRAG),
                PipelineStageConfig::vertex(shaders::YAKUI_VERT),
            ],
            color_formats: vec![color_format],
            blend_enabled: true,
            color_blend: Some(BlendConfig::premultiplied_alpha()),
            alpha_blend: Some(BlendConfig::premultiplied_alpha()),
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

            index_buffer,
            vertex_buffer,

            target_image,

            ui_frame,
        })
    }
}

pub struct UiRenderPassData {
    indices: BufferRange,
    vertices: BufferRange,

    ui_draw_layers: Vec<UiDrawLayer>,
}

impl Pass for UiPass {
    type PassData = UiRenderPassData;

    fn name(&self) -> String {
        String::from("ui")
    }
    
    fn is_enabled(&self, _data_scope: &DataResourceScope) -> bool {
        true
    }

    fn prepare_data(
        &self,
        data_scope: &mut DataResourceScope,
        buffer_scope: &mut BufferResourceScope,
        allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        let ui_frame = data_scope.get(self.ui_frame);

        self.index_buffer.stage_slice(buffer_scope, allocator, &ui_frame.indices)?;
        self.vertex_buffer.stage_slice(buffer_scope, allocator, &ui_frame.vertices)?;

        let indices = buffer_scope.get_physical_buffer(self.index_buffer);
        let vertices = buffer_scope.get_physical_buffer(self.vertex_buffer);

        Ok(UiRenderPassData {
            indices: indices.range,
            vertices: vertices.range,

            ui_draw_layers: ui_frame.draw_layers.clone(),
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .consume(self.ui_frame)
            .write_buffer(
                self.index_buffer,
                AccessFlags::HOST_WRITE,
                PipelineStageFlags::HOST,
            )
            .write_buffer(
                self.vertex_buffer,
                AccessFlags::HOST_WRITE,
                PipelineStageFlags::HOST,
            )
            .read_buffer(
                self.index_buffer,
                AccessFlags::INDEX_READ,
                PipelineStageFlags::VERTEX_INPUT,
            )
            .read_buffer(
                self.vertex_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::VERTEX_SHADER,
            )
            .read_image(
                self.target_image,
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                AccessFlags::COLOR_ATTACHMENT_READ,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
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
            color: vec![ColorTarget { image: self.target_image, mip: None, clear: None }],
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
        let target_image = image_scope.get_physical_image(self.target_image);

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.bind_index_buffer(data.indices, data.indices.offset);

        data.ui_draw_layers.iter().for_each(|draw_layer| {
            draw_layer.draw_calls.iter().for_each(|draw_call| {
                if let Some(clip_area) = &draw_call.clip {
                    context.set_scissor(
                        Offset2D { x: clip_area.position[0], y: clip_area.position[1] },
                        Extent2D { width: clip_area.size[0], height: clip_area.size[1] },
                    );
                } else {
                    context.set_scissor(Offset2D { x: 0, y: 0 }, target_image.extent);
                }

                context.push_constants(
                    self.pipeline_layout,
                    &UiPushConstants::create(
                        data.vertices,
                        draw_call.texture_index,
                        draw_call.render_mode as u32,
                    ),
                );

                context.draw_indexed(
                    draw_call.index_count,
                    draw_call.index_offset,
                    draw_call.vertex_offset,
                );
            });
        });

        Ok(())
    }
    
    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        info!("UiPass destroyed");

        Ok(())
    }
}
