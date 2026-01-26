use std::collections::HashMap;
use std::sync::Arc;
use yakui::{ManagedTextureId, Rect, TextureId, Vec2, Yakui};
use anyhow::Result;
use ash::vk::{Extent2D, Format, SamplerAddressMode, SamplerMipmapMode};
use tracing::warn;
use yakui::paint::{TextureChange, TextureFormat};
use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::buffer::typed::ui_vertex_buffer::UiVertex;
use crate::render::vulkan::render_pass::ui_render_pass::ui_snapshot::{RenderMode, UiDrawCall, UiSnapshot};
use crate::render::vulkan::resource_context::ResourceContext;
use crate::resources::common::resource_provider::ResourceProvider;
use crate::resources::image::image_backend::ImageBackend;
use crate::resources::image::image_config::{ImageConfig, ImageSource};
use crate::resources::res_ref::ResRef;
use crate::resources::resource_hub::ResourceHub;
use crate::resources::sampler::sampler_config::SamplerConfig;
use crate::system_stats::SystemStats;
use crate::ui::ui_renderer::UiRenderer;

pub struct UiContext {
    handle: Yakui,

    ui_renderer: Box<dyn UiRenderer>,

    image_provider: Arc<ResourceProvider<ImageBackend>>,

    buffer_manager: Arc<BufferManager>,

    texture_map: HashMap<ManagedTextureId, u32>,
}

impl UiContext {
    pub fn new(
        resource_context: &ResourceContext,
        resource_hub: Arc<ResourceHub>,
        ui_renderer: Box<dyn UiRenderer>,
    ) -> Result<Self> {
        let handle = Yakui::new();

        Ok(Self {
            handle,

            ui_renderer,

            image_provider: resource_hub.get_image_provider().clone(),

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

        paint_dom.texture_edits().for_each(|(id, change)| {
            match change {
                TextureChange::Added => {
                    println!("TextureChange::Added, {:?}", id);
                }
                TextureChange::Removed => {
                    println!("TextureChange::Removed, {:?}", id);
                }
                TextureChange::Modified => {
                    println!("TextureChange::Modified, {:?}", id);

                    let texture = paint_dom.texture(id).unwrap();

                    let unique_name = format!("yakui_{:?}", id) ;

                    let size = texture.size();

                    let format = match texture.format() {
                        TextureFormat::R8 => Format::R8_UNORM,
                        TextureFormat::Rgba8Srgb => Format::R8G8B8A8_UNORM,
                        _ => unimplemented!(),
                    };

                    let mut texture_resref = ResRef::from(ImageConfig {
                        name: unique_name,
                        source: ImageSource::Virtual {
                            data: texture.data().to_vec(),

                            width: size.x,
                            height: size.y,

                            format,
                        },
                        sampler_config: SamplerConfig {
                            address_mode_u: SamplerAddressMode::CLAMP_TO_EDGE,
                            address_mode_v: SamplerAddressMode::CLAMP_TO_EDGE,

                            mipmap_mode: SamplerMipmapMode::NEAREST,

                            min_lod: 0.0,
                            max_lod: 0.0,

                            mip_lod_bias: 0.0,

                            anisotropy_enable: false,
                            ..SamplerConfig::default()
                        },
                    });

                    if let Some(resource_id) = self.texture_map.get(&id) {
                        self.image_provider.evict(*resource_id);
                    }

                    self.image_provider.ensure(&mut texture_resref);
                    let texture_index = texture_resref.get_id().unwrap();
                    self.image_provider.get_ready(&texture_index);

                    self.texture_map.insert(id, texture_index);
                }
            }
        });

        let mut draw_calls = Vec::new();

        let mut indices = Vec::new();
        let mut vertices = Vec::new();

        paint_dom.layers().iter().for_each(|layer| {
            layer.calls.iter().for_each(|call| {
                let (texture_index, render_mode) = if let Some(texture_id) = call.texture {
                    match texture_id {
                        TextureId::Managed(managed_texture_id) => {
                            let texture_id = self.texture_map
                                .get(&managed_texture_id)
                                .expect("Texture id expected, but not found");

                            (*texture_id, RenderMode::Texture)
                        }
                        TextureId::User(_id) => {
                            warn!("User images are not yet supported");

                            return;
                        }
                    }
                } else {
                    (0, RenderMode::Solid)
                };

                draw_calls.push(UiDrawCall {
                    index_count: call.indices.len(),
                    index_offset: indices.len(),
                    vertex_offset: vertices.len(),

                    texture_index,
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
