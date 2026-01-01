use crate::assembler::adapter::adapter::ResourceAdapter;
use crate::assembler::adapter::mesh_adapter::MeshAdapter;
use crate::assembler::adapter::model_adapter::{ModelAdapter, ModelResource};
use crate::assembler::adapter::primitive_adapter::PrimitiveAdapter;
use crate::assembler::collector::collector::ResourceCollector;
use crate::assembler::collector::image_collector::ImageCollector;
use crate::assembler::key_generator::ResourceKeyGenerator;
use crate::assembler::models::meshopt_utils::optimize_model;
use crate::assembler::utils::write_bytes;
use crate::data::common::image_data::ImageData;
use crate::data::common::model_data::ModelData;
use anyhow::Result;
use gltf::import;
use rkyv::rancor::Error;
use rkyv::to_bytes;
use std::fs::create_dir_all;
use std::path::Path;
use std::sync::{Arc, Mutex};
use crate::assembler::collector::material_collector::MaterialCollector;
use crate::data::common::material_data::MaterialData;

pub struct ModelPipeline {
    material_collector: Arc<Mutex<MaterialCollector>>,
    image_collector: Arc<Mutex<ImageCollector>>,

    model_adapter: ModelAdapter,
}

impl ModelPipeline {
    pub fn new() -> Self {
        let textures_key_generator = Arc::new(ResourceKeyGenerator::create());

        let image_collector = {
            let collector = ImageCollector::create(textures_key_generator.clone());

            Arc::new(Mutex::new(collector))
        };
        let material_collector = {
            let collector = MaterialCollector::create(
                textures_key_generator.clone(),
                image_collector.clone(),
            );

            Arc::new(Mutex::new(collector))
        };
        let primitive_adapter = PrimitiveAdapter::create(material_collector.clone());
        let mesh_adapter = MeshAdapter::create(primitive_adapter);
        let model_adapter = ModelAdapter::create(mesh_adapter);

        Self {
            material_collector,
            image_collector,

            model_adapter,
        }
    }

    pub fn collect_models(
        &mut self,
        source_path: &Path,
        generated_root_path: &Path,
        file_path: &Path,
        collection_name: &str,
    ) -> Result<()> {
        let result_path = generated_root_path.join(file_path);

        println!("Adapting GLB: {:?}", source_path.display());

        let (document, buffers, _images) = import(&source_path)?;

        let collection_node = document.nodes().find(|node| {
            let name = node.name().unwrap().to_string();

            name == collection_name
        });

        let Some(collection_node) = collection_node else { return Ok(()) };

        let mut model_data = self.model_adapter.adapt(&ModelResource {
            collection_node,

            local_path: &file_path,

            buffers: &buffers,
        })?;

        optimize_model(&mut model_data)?;

        Self::write_manifest(&result_path, &model_data, collection_name)?;

        let mut image_collector = self.image_collector.lock().unwrap();
        let images = image_collector.get_resources();
        Self::write_textures(&generated_root_path, &images)?;
        image_collector.reset();

        let mut material_collector = self.material_collector.lock().unwrap();
        let materials = material_collector.get_resources();
        Self::write_materials(&generated_root_path, &materials)?;
        material_collector.reset();

        Ok(())
    }

    fn write_manifest(target_path: &Path, model_data: &ModelData, name: &str) -> Result<()> {
        create_dir_all(&target_path)?;

        let result_model_path = target_path.join(name).with_extension("manifest");

        let model_bytes = to_bytes::<Error>(model_data)?.into_vec();

        write_bytes(&result_model_path, &model_bytes)?;

        Ok(())
    }

    fn write_materials(target_path: &Path, materials: &[(String, &MaterialData)]) -> Result<()> {
        for (key, material_data) in materials {
            let material_path = target_path.join(key).with_extension("material");

            let material_bytes = to_bytes::<Error>(*material_data)?.into_vec();

            write_bytes(&material_path, &material_bytes)?;
        }

        Ok(())
    }

    fn write_textures(target_path: &Path, images: &[(String, &ImageData)]) -> Result<()> {
        for (key, image_data) in images {
            let texture_path = target_path.join(key);

            write_bytes(&texture_path, &image_data.data)?;
        }

        Ok(())
    }
}
