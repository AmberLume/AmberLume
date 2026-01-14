use std::sync::Arc;
use yakui::{align, colored_box, Alignment, Color, Vec2, Yakui};
use anyhow::Result;
use ash::vk::Extent2D;
use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::buffer::typed::ui_vertex_buffer::UiVertex;
use crate::render::vulkan::render_pass::ui_render_pass::ui_snapshot::{UiDrawCall, UiSnapshot};
use crate::render::vulkan::resource_context::ResourceContext;

pub struct UiContext {
    handle: Yakui,

    buffer_manager: Arc<BufferManager>,
}

impl UiContext {
    pub fn new(
        resource_context: &ResourceContext,
    ) -> Result<Self> {
        let handle = Yakui::new();

        Ok(Self {
            handle,

            buffer_manager: resource_context.buffer_manager.clone(),
        })
    }

    pub fn render_base_ui(&mut self, extent: Extent2D) {
        let size = Vec2::new(extent.width as f32, extent.height as f32);
        self.handle.set_surface_size(size);

        self.handle.start();

        align(Alignment::TOP_LEFT, || {
            colored_box(Color::BLUE, [100.0, 100.0]);
            colored_box(Color::GREEN, [50.0, 50.0]);
        });

        self.handle.finish();
    }

    pub fn build_ui_snapshot(&mut self) -> Result<UiSnapshot> {
        let paint_dom = self.handle.paint();

        let mut draw_calls = Vec::new();

        let mut indices = Vec::new();
        let mut vertices = Vec::new();

        paint_dom.layers().iter().for_each(|layer| {
            layer.calls.iter().for_each(|call| {
                draw_calls.push(UiDrawCall {
                    index_count: call.indices.len(),
                    index_offset: indices.len(),
                    vertex_offset: vertices.len(),
                });

                indices.extend(call.indices.iter().map(|i| *i as u32));
                vertices.extend(call.vertices.iter().map(|vertex| {
                    UiVertex {
                        position: vertex.position.to_array(),
                        tex_coords: vertex.texcoord.to_array(),
                        color: vertex.color.to_array(),
                    }
                }));
            });
        });

        self.buffer_manager.ui_index_buffer.stage(0, &indices)?;
        self.buffer_manager.ui_vertex_buffer.stage(0, &vertices)?;

        Ok(UiSnapshot {
            draw_calls,
        })
    }

    pub fn destroy(self) -> Result<()> {
        drop(self.buffer_manager);

        Ok(())
    }
}
