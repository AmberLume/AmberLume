use crate::render::render_graph::pass::Pass;
use crate::render::pass::pass_context::PassContext;
use crate::render::pass::pass_resources::PassResources;
use anyhow::{bail, Result};
use ash::vk::{AccessFlags, Buffer, DeviceAddress, DeviceSize, Format, ImageLayout, Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags};
use std::sync::Arc;
use tracing::info;
use gpu::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::ui::ui_push_constants::UiPushConstants;
use crate::render::pass::ui::ui_frame::UiDrawLayer;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_image::render_targets::{ColorTarget, RenderTargets};
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;
use crate::resources::store::providers::res_ref::ResRef;
use gpu::PipelineLayoutType;
use crate::resources::store::providers::pipeline::pipeline_config::{BlendConfig, PipelineConfig, PipelineStageConfig};
use crate::resources::resource_manifest::shaders;

pub struct UiPass {
    _handle: Arc<ResRef>,
    
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,

    index_buffer: VirtualBuffer,
    vertex_buffer: VirtualBuffer,

    target_image: VirtualImage,
}

impl UiPass {
    pub fn create(
        resources: &PassResources,
        index_buffer: VirtualBuffer,
        vertex_buffer: VirtualBuffer,
        color_format: Format,
        target_image: VirtualImage,
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

        let _handle = resources.pipeline_provider.acquire_sync(pipeline_config);
        let Some(pipeline) = resources.pipeline_provider.get_resource(_handle.id) else {
            bail!("Failed to acquire Pipeline");
        };

        Ok(Self {
            _handle,

            pipeline: *pipeline,
            pipeline_layout: resources.pipeline_layout_registry.get(PipelineLayoutType::General),

            index_buffer,
            vertex_buffer,

            target_image,
        })
    }
}

pub struct UiRenderPassData {
    indices_handle: Buffer,
    indices_offset: DeviceSize,

    vertices: DeviceAddress,

    ui_draw_layers: Vec<UiDrawLayer>,
}

impl Pass for UiPass {
    type PassData = UiRenderPassData;

    fn name(&self) -> String {
        String::from("ui")
    }
    
    fn is_enabled(&self, _context: &FrameDataContext) -> bool {
        true
    }

    fn prepare_data(
        &self,
        context: &FrameDataContext,
        buffer_scope: &mut BufferResourceScope,
        allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        self.index_buffer.stage_slice(buffer_scope, allocator, &context.ui_frame.indices)?;
        self.vertex_buffer.stage_slice(buffer_scope, allocator, &context.ui_frame.vertices)?;

        let indices = buffer_scope.get_physical_buffer(self.index_buffer);
        let vertices = buffer_scope.get_physical_buffer(self.vertex_buffer);

        Ok(UiRenderPassData {
            indices_handle: indices.buffer,
            indices_offset: indices.offset,

            vertices: vertices.device_address,

            ui_draw_layers: context.ui_frame.draw_layers.clone(),
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
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

    fn record_commands(&self, context: &PassContext, image_scope: &ImageResourceScope, _buffer_scope: &BufferResourceScope, data: Self::PassData) -> Result<()> {
        let target_image = image_scope.get_physical_image(self.target_image);

        context.bind_pipeline(PipelineBindPoint::GRAPHICS, self.pipeline);

        context.bind_ui_index_buffer(data.indices_handle, data.indices_offset);

        data.ui_draw_layers.iter().for_each(|draw_layer| {
            draw_layer.draw_calls.iter().for_each(|draw_call| {
                if let Some(clip_area) = &draw_call.clip {
                    context.set_area_scissor(&clip_area);
                } else {
                    context.set_image_scissor(&target_image);
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
