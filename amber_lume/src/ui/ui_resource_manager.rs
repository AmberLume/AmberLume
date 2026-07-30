use gpu::ImageDescription;
use gpu::ImageViewDescription;
use std::collections::HashMap;
use std::sync::Arc;
use ash::vk::{Extent3D, Format};
use yakui::{ManagedTextureId, TextureId};
use yakui::paint::{PaintDom, TextureChange, TextureFormat};
use anyhow::Result;
use tracing::warn;
use crate::render::pass::ui::ui_frame::UiVertex;
use crate::render::pass::ui::ui_frame::{ClipArea, RenderMode, UiDrawCall, UiDrawLayer, UiFrame};
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::store::providers::resource_provider::ResourceProvider;
use crate::resources::store::persistent::persistent_resources::PersistentResources;
use crate::resources::store::providers::image::image_backend::ImageBackend;
use crate::resources::store::providers::image::image_config::ImageConfig;

pub struct UiResourceManager {
    persistent_resources: Arc<PersistentResources>,

    image_provider: Arc<ResourceProvider<ImageBackend>>,

    texture_map: HashMap<ManagedTextureId, Arc<ResRef>>,
}

impl UiResourceManager {
    pub fn new(
        persistent_resources: Arc<PersistentResources>,
        image_provider: Arc<ResourceProvider<ImageBackend>>,
    ) -> Self {
        Self {
            persistent_resources,

            image_provider,

            texture_map: HashMap::new(),
        }
    }

    pub fn collect_resource_edits(
        &mut self,
        paint_dom: &PaintDom,
    ) -> Result<()> {
        for (id, change) in paint_dom.texture_edits() {
            match change {
                TextureChange::Added | TextureChange::Modified => {
                    self.add_texture(paint_dom, id);
                }
                TextureChange::Removed => {
                    self.texture_map.remove(&id);
                }
            }
        }

        Ok(())
    }

    pub fn fill_draw_buffers(&self, paint_dom: &PaintDom) -> Result<UiFrame> {
        let mut draw_layers = Vec::new();

        let mut indices = Vec::new();
        let mut vertices = Vec::new();

        for layer in paint_dom.layers().iter() {
            let mut draw_calls = Vec::new();

            for call in layer.calls.iter() {
                let (image, render_mode) = if let Some(texture_id) = call.texture {
                    match texture_id {
                        TextureId::Managed(managed_texture_id) => {
                            if let Some(image) = self.get_texture(&managed_texture_id) {
                                (image, RenderMode::Texture)
                            } else {
                                (self.persistent_resources.images.white_pixel.clone(), RenderMode::Texture)
                            }
                        }
                        TextureId::User(_id) => {
                            warn!("User images are not yet supported");

                            continue;
                        }
                    }
                } else {
                    (self.persistent_resources.images.white_pixel.clone(), RenderMode::Solid)
                };

                draw_calls.push(UiDrawCall {
                    index_count: call.indices.len(),
                    index_offset: indices.len(),
                    vertex_offset: vertices.len(),

                    clip: call.clip.map(|clip| {
                        ClipArea {
                            position: clip.pos().to_array().map(|x| x as i32),
                            size: clip.size().to_array().map(|x| x as u32),
                        }
                    }),

                    texture_index: image.id,
                    render_mode,
                });

                indices.extend(call.indices.iter().map(|i| *i as u32));
                vertices.extend(call.vertices.iter().map(|vertex| {
                    UiVertex {
                        position: vertex.position.to_array(),
                        tex_coords: vertex.texcoord.to_array(),
                        color: vertex.color.to_array(),
                    }
                }));
            }

            draw_layers.push(UiDrawLayer {
                draw_calls,
            })
        }

        Ok(UiFrame {
            indices,
            vertices,

            draw_layers,
        })
    }

    fn add_texture(&mut self, paint_dom: &PaintDom, id: ManagedTextureId) {
        let unique_name = format!("yakui_{:?}", id);

        let texture = paint_dom.texture(id).expect("Texture must exist");
        let size = texture.size();

        let extent = Extent3D {
            width: size.x,
            height: size.y,
            depth: 1,
        };

        let format = match texture.format() {
            TextureFormat::R8 => Format::R8_UNORM,
            TextureFormat::Rgba8Srgb => Format::R8G8B8A8_UNORM,
            _ => unimplemented!(),
        };

        let image = self.image_provider.acquire_sync(ImageConfig::Inbuilt {
            label: unique_name,

            image_description: ImageDescription::default(
                format,
                extent,
            ),
            image_view_description: ImageViewDescription::default_2d_color(),

            data: Some(texture.data().to_vec()),
        });

        self.texture_map.insert(id, image);
    }

    pub fn get_texture(&self, managed_texture_id: &ManagedTextureId) -> Option<Arc<ResRef>> {
        if let Some(image) = self.texture_map.get(&managed_texture_id) {
            Some(image.clone())
        } else {
            warn!("Texture id expected, but not found");

            None
        }
    }

    pub fn destroy(mut self) {
        self.texture_map.clear();
    }
}
