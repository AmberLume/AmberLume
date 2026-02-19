use std::collections::HashMap;
use std::sync::Arc;
use yakui::{ManagedTextureId, Rect, TextureId, Vec2, Yakui};
use anyhow::Result;
use ash::vk::{Extent2D, Extent3D, Format, ImageAspectFlags, ImageSubresourceLayers};
use tracing::warn;
use yakui::paint::{TextureChange, TextureFormat};
use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::buffer::typed::ui_vertex_buffer::UiVertex;
use crate::render::vulkan::factories::image::managed_image::{ImageDescription, ImageViewDescription, ManagedImage};
use crate::render::vulkan::render_pass::ui_render_pass::ui_snapshot::{RenderMode, UiDrawCall, UiSnapshot};
use crate::render::vulkan::resource_loader::ResourceLoader;
use crate::resources::descriptor_index_managers::DescriptorIndexManagers;
use crate::resources::dynamic::resource_provider::ResourceId;
use crate::resources::persistent::persistent_resources::PersistentResources;
use crate::resources::resource_factories::ResourceFactories;
use crate::system_stats::SystemStats;
use crate::ui::ui_renderer::UiRenderer;
pub struct UiContext {
    handle: Yakui,

    resource_factories: Arc<ResourceFactories>,
    index_managers: Arc<DescriptorIndexManagers>,
    persistent_resources: Arc<PersistentResources>,

    ui_renderer: Box<dyn UiRenderer>,

    buffer_manager: Arc<BufferManager>,

    resource_loader: Arc<ResourceLoader>,

    texture_map: HashMap<ManagedTextureId, (ResourceId, ManagedImage)>,
}

impl UiContext {
    pub fn new(
        resource_factories: Arc<ResourceFactories>,
        index_managers: Arc<DescriptorIndexManagers>,
        persistent_resources: Arc<PersistentResources>,
        buffer_manager: Arc<BufferManager>,
        resource_loader: Arc<ResourceLoader>,
        ui_renderer: Box<dyn UiRenderer>,
    ) -> Result<Self> {
        let handle = Yakui::new();

        Ok(Self {
            handle,

            resource_factories,
            index_managers,
            persistent_resources,

            ui_renderer,

            buffer_manager,

            resource_loader,

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

    pub fn build_ui_snapshot(
        &mut self,
        current_frame: u64,
    ) -> Result<UiSnapshot> {
        let paint_dom = self.handle.paint();

        for (id, change) in paint_dom.texture_edits() {
            match change {
                TextureChange::Added | TextureChange::Modified => {
                    let unique_name = format!("yakui_{:?}", id) ;

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

                    let (image_resource_id, managed_image) = if let Some((resource_id, image)) = self.texture_map.remove(&id) {
                        if image.image_description.extent == extent {
                            (resource_id, image)
                        } else {
                            self.resource_factories.managed_image_factory.destroy_image(image)?;

                            let managed_image = self.resource_factories.managed_image_factory.allocate(
                                &unique_name,
                                ImageDescription::default(
                                    format,
                                    extent,
                                ),
                                ImageViewDescription::default_2d_color(),
                            )?;

                            (resource_id, managed_image)
                        }
                    } else {
                        let resource_id = self.index_managers.image_index_manager.acquire().unwrap();

                        let managed_image = self.resource_factories.managed_image_factory.allocate(
                            &unique_name,
                            ImageDescription::default(
                                format,
                                extent,
                            ),
                            ImageViewDescription::default_2d_color(),
                        )?;

                        (resource_id, managed_image)
                    };

                    self.resource_loader.load_image(
                        managed_image.image,
                        extent,
                        ImageSubresourceLayers {
                            aspect_mask: ImageAspectFlags::COLOR,
                            mip_level: 0,
                            base_array_layer: 0,
                            layer_count: 1,
                        },
                        1,
                        1,
                        texture.data(),
                    )?;

                    self.persistent_resources.descriptor_sets.global.bind_image(
                        image_resource_id,
                        &managed_image,
                        self.persistent_resources.samplers.linear_clamp,
                    );

                    self.texture_map.insert(id, (image_resource_id, managed_image));
                }
                TextureChange::Removed => {
                    let (resource_id, managed_image) = self.texture_map.remove(&id).unwrap();

                    self.resource_factories.managed_image_factory.destroy_image(managed_image)?;
                    self.index_managers.image_index_manager.release(resource_id, current_frame);
                }
            }
        };

        let mut draw_calls = Vec::new();

        let mut indices = Vec::new();
        let mut vertices = Vec::new();

        for layer in paint_dom.layers().iter() {
            for call in layer.calls.iter() {
                let (image_descriptor_id, render_mode) = if let Some(texture_id) = call.texture {
                    match texture_id {
                        TextureId::Managed(managed_texture_id) => {
                            let (image_descriptor_id, _) = self.texture_map
                                .get(&managed_texture_id)
                                .expect("Texture id expected, but not found");

                            (*image_descriptor_id, RenderMode::Texture)
                        }
                        TextureId::User(_id) => {
                            warn!("User images are not yet supported");

                            continue;
                        }
                    }
                } else {
                    (self.persistent_resources.images.white_pixel.descriptor_index, RenderMode::Solid)
                };

                draw_calls.push(UiDrawCall {
                    index_count: call.indices.len(),
                    index_offset: indices.len(),
                    vertex_offset: vertices.len(),

                    texture_index: image_descriptor_id,
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
            };
        };

        self.buffer_manager.ui_index_buffer.replace_with(&indices)?;
        self.buffer_manager.ui_vertex_buffer.replace_with(&vertices)?;

        Ok(UiSnapshot {
            draw_calls,
        })
    }

    pub fn destroy(self) -> Result<()> {
        for (_, (_, managed_image)) in self.texture_map {
            self.resource_factories.managed_image_factory.destroy_image(managed_image)?;
        }

        Ok(())
    }
}
