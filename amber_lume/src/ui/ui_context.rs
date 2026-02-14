use std::collections::HashMap;
use std::sync::Arc;
use yakui::{ManagedTextureId, Rect, Vec2, Yakui};
use anyhow::Result;
use ash::vk::Extent2D;
use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::factories::image::managed_image::ManagedImage;
use crate::render::vulkan::render_pass::ui_render_pass::ui_snapshot::{UiSnapshot};
use crate::render::vulkan::resource_context::ResourceContext;
use crate::resources::resource_factories::ResourceFactories;
use crate::system_stats::SystemStats;
use crate::ui::ui_renderer::UiRenderer;

pub struct UiContext {
    handle: Yakui,

    resource_factories: Arc<ResourceFactories>,

    ui_renderer: Box<dyn UiRenderer>,

    buffer_manager: Arc<BufferManager>,

    texture_map: HashMap<ManagedTextureId, ManagedImage>,
}

impl UiContext {
    pub fn new(
        resource_context: &ResourceContext,
        resource_factories: Arc<ResourceFactories>,
        ui_renderer: Box<dyn UiRenderer>,
    ) -> Result<Self> {
        let handle = Yakui::new();

        Ok(Self {
            handle,

            resource_factories: resource_factories.clone(),

            ui_renderer,

            buffer_manager: resource_context.buffer_manager.clone(),

            texture_map: HashMap::new(),
        })
    }

    pub fn render_ui(
        &mut self,
        extent: Extent2D,
        system_stats: &Option<SystemStats>,
    ) {
        let size = Vec2::new(extent.width as f32, extent.height as f32);

        self.handle.set_surface_size(size);
        self.handle.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, size));

        self.handle.start();

        self.ui_renderer.render(&system_stats);

        self.handle.finish();
    }

    pub fn build_ui_snapshot(&mut self) -> Result<UiSnapshot> {
        let paint_dom = self.handle.paint();

        // for (id, change) in paint_dom.texture_edits() {
        //     match change {
        //         TextureChange::Added => {
        //             println!("TextureChange::Added, {:?}", id);
        //         }
        //         TextureChange::Removed => {
        //             let managed_image = self.texture_map.remove(&id).unwrap();
        //
        //             self.managed_image_factory.destroy(managed_image)?;
        //
        //             println!("TextureChange::Removed, {:?}", id);
        //         }
        //         TextureChange::Modified => {
        //             println!("TextureChange::Modified, {:?}", id);
        //
        //             let texture = paint_dom.texture(id).unwrap();
        //
        //             let unique_name = format!("yakui_{:?}", id) ;
        //
        //             let size = texture.size();
        //
        //             let format = match texture.format() {
        //                 TextureFormat::R8 => Format::R8_UNORM,
        //                 TextureFormat::Rgba8Srgb => Format::R8G8B8A8_UNORM,
        //                 _ => unimplemented!(),
        //             };
        //
        //             let image_description = ImageDescription {
        //                 image_type: ImageType::TYPE_2D,
        //                 format,
        //                 extent: Extent3D {
        //                     width: size.x,
        //                     height: size.y,
        //                     depth: 1,
        //                 },
        //                 mip_levels: 1,
        //                 array_layers: 1,
        //                 samples: SampleCountFlags::TYPE_1,
        //                 tiling: ImageTiling::OPTIMAL,
        //                 usage: ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
        //                 sharing_mode: SharingMode::EXCLUSIVE,
        //             };
        //             let image_view_description = ImageViewDescription {
        //                 image_view_type: ImageViewType::TYPE_2D,
        //                 image_aspect_flags: ImageAspectFlags::COLOR,
        //                 base_mip_level: 0,
        //                 level_count: 1,
        //                 base_array_layer: 0,
        //                 layer_count: 1,
        //             };
        //
        //             let managed_image = self.managed_image_factory.allocate(
        //                 &unique_name,
        //                 image_description,
        //                 image_view_description,
        //             );
        //
        //             if let Ok(managed_image) = managed_image {
        //                 self.texture_map.insert(id, managed_image);
        //             } else {
        //                 warn!("Failed to allocate ManagedImage for UiContext");
        //             }
        //         }
        //     }
        // };
        //
        let mut draw_calls = Vec::new();
        //
        // let mut indices = Vec::new();
        // let mut vertices = Vec::new();
        //
        // for layer in paint_dom.layers().iter() {
        //     for call in layer.calls.iter() {
        //         let (managed_image, render_mode) = if let Some(texture_id) = call.texture {
        //             match texture_id {
        //                 TextureId::Managed(managed_texture_id) => {
        //                     let managed_image = self.texture_map
        //                         .get(&managed_texture_id)
        //                         .expect("Texture id expected, but not found");
        //
        //                     (*managed_image, RenderMode::Texture)
        //                 }
        //                 TextureId::User(_id) => {
        //                     warn!("User images are not yet supported");
        //
        //                     continue;
        //                 }
        //             }
        //         } else {
        //             continue;
        //         };
        //
        //         draw_calls.push(UiDrawCall {
        //             index_count: call.indices.len(),
        //             index_offset: indices.len(),
        //             vertex_offset: vertices.len(),
        //
        //             texture_index: managed_image,
        //             render_mode,
        //         });
        //
        //         indices.extend(call.indices.iter().map(|i| *i as u32));
        //         vertices.extend(call.vertices.iter().map(|vertex| {
        //             UiVertex {
        //                 position: vertex.position.to_array(),
        //                 tex_coords: vertex.texcoord.to_array(),
        //                 color: vertex.color.to_array(),
        //             }
        //         }));
        //     };
        // };
        //
        // self.buffer_manager.ui_index_buffer.stage(0, &indices)?;
        // self.buffer_manager.ui_vertex_buffer.stage(0, &vertices)?;

        Ok(UiSnapshot {
            draw_calls,
        })
    }

    pub fn destroy(self) -> Result<()> {
        drop(self.buffer_manager);

        Ok(())
    }
}
