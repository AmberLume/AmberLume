use crate::assembler::collector::collector::ResourceCollector;
use crate::assembler::key_generator::ResourceKeyGenerator;
use anyhow::{Result};
use gltf::{Material};
use gltf::buffer::Data;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use crate::assembler::collector::image_collector::{ImageCollector, ImageResource};
use crate::data::common::material_data::MaterialData;

pub struct MaterialCollector {
    image_collector: Arc<Mutex<ImageCollector>>,

    key_generator: Arc<ResourceKeyGenerator>,

    resources: HashMap<String, MaterialData>,
}

impl MaterialCollector {
    pub fn create(
        key_generator: Arc<ResourceKeyGenerator>,
        image_collector: Arc<Mutex<ImageCollector>>,
    ) -> Self {
        Self {
            image_collector,

            key_generator,

            resources: HashMap::new(),
        }
    }
}

pub struct MaterialResource<'a> {
    pub material: Material<'a>,

    pub local_path: &'a Path,

    pub buffers: &'a [Data],
}

impl ResourceCollector for MaterialCollector {
    type Input<'a> = MaterialResource<'a>;

    type Output = MaterialData;

    fn collect<'a>(&mut self, input: &Self::Input<'a>) -> Result<String> {
        let name = input
            .material
            .name()
            .expect("Material names are required")
            .to_owned();
        let key = input.local_path.join(self.key_generator.get_next_key()).with_extension("material").to_str().unwrap().to_string();
        println!("Collecting material '{}' as '{}'...", name, key);

        let pbr_metallic_roughness = input.material.pbr_metallic_roughness();
        let base_color = pbr_metallic_roughness.base_color_factor();
        let base_texture_info = pbr_metallic_roughness.base_color_texture();

        let mut base_texture_id: Option<String> = None;
        match base_texture_info {
            Some(texture_info) => {
                let texture = texture_info.texture();
                let image = texture.source();

                let mut image_collector = self.image_collector.lock().unwrap();

                let base_texture_key = image_collector.collect(&ImageResource {
                    image,

                    is_srgb: true,

                    local_path: input.local_path,

                    buffers: input.buffers,
                })?;

                base_texture_id = Some(base_texture_key);
            }
            None => {
                println!("No base texture");
            }
        }

        self.resources.insert(
            key.clone(),
            MaterialData {
                base_color,
                base_texture_id,
            },
        );

        Ok(key)
    }

    fn get_resources(&self) -> Vec<(String, &Self::Output)> {
        self.resources
            .iter()
            .map(|(key, value)| (key.clone(), value))
            .collect()
    }

    fn reset(&mut self) {
        self.resources.clear();
    }
}
