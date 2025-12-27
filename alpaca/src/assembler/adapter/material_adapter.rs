use crate::assembler::adapter::adapter::ResourceAdapter;
use crate::assembler::collector::collector::ResourceCollector;
use crate::assembler::collector::image_collector::{ImageCollector, ImageResource};
use crate::data::common::material_data::MaterialData;
use anyhow::Result;
use gltf::Material;
use gltf::buffer::Data;
use std::sync::{Arc, Mutex};

pub struct MaterialAdapter {
    image_collector: Arc<Mutex<ImageCollector>>,
}

impl MaterialAdapter {
    pub fn create(image_collector: Arc<Mutex<ImageCollector>>) -> Self {
        Self { image_collector }
    }
}

pub struct MaterialResource<'a> {
    pub material: Material<'a>,

    pub buffers: &'a [Data],
}

impl ResourceAdapter for MaterialAdapter {
    type Input<'a> = MaterialResource<'a>;

    type Output = MaterialData;

    fn adapt<'a>(&mut self, input: &Self::Input<'a>) -> Result<Self::Output> {
        let name = input
            .material
            .name()
            .expect("Material names are required")
            .to_owned();
        println!("Collecting material '{}'...", name);

        let base_texture_info = input.material.pbr_metallic_roughness().base_color_texture();

        let mut base_texture: Option<String> = None;

        match base_texture_info {
            Some(texture_info) => {
                let texture = texture_info.texture();
                let image = texture.source();

                let mut image_collector = self.image_collector.lock().unwrap();

                let base_texture_key = image_collector.collect(&ImageResource {
                    image,

                    is_srgb: true,

                    buffers: input.buffers,
                })?;

                base_texture = Some(base_texture_key);
            }
            None => {
                println!("No base texture");
            }
        }

        Ok(Self::Output {
            base_texture_key: base_texture,
        })
    }
}
