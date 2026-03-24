use crate::render::buffer::buffer_manager::BufferManager;
use crate::render::factories::image::managed_image::{ImageDescription, ImageViewDescription, ManagedImage};
use crate::render::resources::resource_loader::ResourceLoader;
use crate::resources::descriptor_index_managers::IndexManagers;
use crate::resources::persistent::persistent_resources::PersistentResources;
use crate::resources::resource_factories::ResourceFactories;
use std::collections::HashMap;
use std::sync::Arc;
use ash::vk::{AccessFlags, Extent3D, Format, ImageAspectFlags, ImageSubresourceLayers};
use yakui::{ManagedTextureId, TextureId};
use yakui::paint::{PaintDom, TextureChange, TextureFormat};
use crate::resources::persistent::persistent_descriptor_set_layouts::GlobalDescriptorSetBindings;
use anyhow::Result;
use tracing::warn;
use crate::ids::{FrameIndex, SliceIndex};
use crate::render::buffer::typed::ui_vertex_buffer::UiVertex;
use crate::render::render_pass::ui::ui_snapshot::{ClipArea, RenderMode, UiDrawCall, UiDrawLayer, UiSnapshot};
use crate::resources::dynamic::resource_provider::ResourceId;

pub struct UiResourceManager {
    resource_factories: Arc<ResourceFactories>,
    index_managers: Arc<IndexManagers>,
    persistent_resources: Arc<PersistentResources>,

    buffer_manager: Arc<BufferManager>,
    resource_loader: Arc<ResourceLoader>,

    texture_map: HashMap<ManagedTextureId, (ResourceId, ManagedImage)>,
}

impl UiResourceManager {
    pub fn new(
        resource_factories: Arc<ResourceFactories>,
        index_managers: Arc<IndexManagers>,
        persistent_resources: Arc<PersistentResources>,
        buffer_manager: Arc<BufferManager>,
        resource_loader: Arc<ResourceLoader>,
    ) -> Self {
        Self {
            resource_factories,
            index_managers,
            persistent_resources,

            buffer_manager,
            resource_loader,

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
                    self.add_texture(paint_dom, id)?;
                }
                TextureChange::Removed => {
                    let (resource_id, managed_image) = self.texture_map.remove(&id).unwrap();

                    self.index_managers.texture_descriptors_index_manager.release(resource_id);
                    self.resource_factories.managed_image_factory.destroy_image(managed_image)?;
                }
            }
        }

        Ok(())
    }

    pub fn fill_draw_buffers(&self, paint_dom: &PaintDom, frame_index: FrameIndex) -> Result<UiSnapshot> {
        let mut draw_layers = Vec::new();

        let mut indices = Vec::new();
        let mut vertices = Vec::new();

        for layer in paint_dom.layers().iter() {
            let mut draw_calls = Vec::new();

            for call in layer.calls.iter() {
                let (image_descriptor_id, render_mode) = if let Some(texture_id) = call.texture {
                    match texture_id {
                        TextureId::Managed(managed_texture_id) => {
                            if let Some(texture_resource_id) = self.get_texture_resource_id(&managed_texture_id) {
                                (texture_resource_id, RenderMode::Texture)
                            } else {
                                (self.get_stub_texture_resource_id(), RenderMode::Texture)
                            }
                        }
                        TextureId::User(_id) => {
                            warn!("User images are not yet supported");

                            continue;
                        }
                    }
                } else {
                    (self.get_stub_texture_resource_id(), RenderMode::Solid)
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
            }

            draw_layers.push(UiDrawLayer {
                draw_calls,
            })
        }

        let _ = self.buffer_manager.ui_index_buffer.frame(frame_index).slice_at(SliceIndex::ZERO).stage(&indices, AccessFlags::empty())?;
        let _ = self.buffer_manager.ui_vertex_buffer.frame(frame_index).slice_at(SliceIndex::ZERO).stage(&vertices, AccessFlags::empty())?;

        Ok(UiSnapshot {
            draw_layers,
        })
    }

    fn add_texture(&mut self, paint_dom: &PaintDom, id: ManagedTextureId) -> Result<()> {
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

        let (resource_id, managed_image) = if let Some((resource_id, managed_image)) = self.texture_map.remove(&id) {
            if managed_image.image_description.extent == extent {
                (resource_id, managed_image)
            } else {
                self.resource_factories.managed_image_factory.destroy_image(managed_image)?;

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
            let resource_id = self.index_managers.texture_descriptors_index_manager.acquire().unwrap();

            let managed_image = self.resource_factories.managed_image_factory.allocate(
                &unique_name,
                ImageDescription::default(
                    format,
                    extent,
                ),
                ImageViewDescription::default_2d_color(),
            )?;

            self.persistent_resources.descriptor_sets.global.bind_image(
                resource_id,
                GlobalDescriptorSetBindings::Texture as u32,
                &managed_image,
                self.persistent_resources.samplers.linear_clamp,
            );

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

        self.texture_map.insert(id, (resource_id, managed_image));

        Ok(())
    }

    pub fn get_texture_resource_id(&self, managed_texture_id: &ManagedTextureId) -> Option<ResourceId> {
        if let Some((resource_id, _)) = self.texture_map.get(&managed_texture_id) {
            Some(*resource_id)
        } else {
            warn!("Texture id expected, but not found");

            None
        }
    }

    pub fn get_stub_texture_resource_id(&self) -> ResourceId {
        self.persistent_resources.images.white_pixel.descriptor_index
    }

    pub fn destroy(self) -> Result<()> {
        for (_, (_, managed_image)) in self.texture_map {
            self.resource_factories.managed_image_factory.destroy_image(managed_image)?;
        }

        Ok(())
    }
}
