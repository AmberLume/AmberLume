use crate::assembler::adapter::adapter::ResourceAdapter;
use crate::assembler::adapter::material_adapter::MaterialAdapter;
use crate::assembler::adapter::mesh_adapter::MeshAdapter;
use crate::assembler::adapter::model_adapter::{ModelAdapter, ModelResource};
use crate::assembler::adapter::primitive_adapter::PrimitiveAdapter;
use crate::assembler::collector::collector::ResourceCollector;
use crate::assembler::collector::image_collector::ImageCollector;
use crate::assembler::key_generator::ResourceKeyGenerator;
use crate::assembler::models::meshopt_utils::optimize_model;
use crate::assembler::resource_pipeline::ResourcePipeline;
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

pub struct ModelPipeline {
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
        let material_adapter = MaterialAdapter::create(image_collector.clone());
        let primitive_adapter = PrimitiveAdapter::create(material_adapter);
        let mesh_adapter = MeshAdapter::create(primitive_adapter);
        let model_adapter = ModelAdapter::create(mesh_adapter);

        Self {
            image_collector,

            model_adapter,
        }
    }

    fn write_model(target_path: &Path, model_data: &ModelData) -> Result<()> {
        create_dir_all(&target_path)?;

        let result_model_path = target_path.join("model");

        let model_bytes = to_bytes::<Error>(model_data)?.into_vec();

        write_bytes(&result_model_path, &model_bytes)?;

        Ok(())
    }

    fn write_textures(target_path: &Path, images: &[(String, &ImageData)]) -> Result<()> {
        for (key, image_data) in images {
            let texture_path = target_path.join(key).with_extension("ktx2");

            write_bytes(&texture_path, &image_data.data)?;
        }

        Ok(())
    }
}

impl ResourcePipeline for ModelPipeline {
    fn can_assemble(&self, extension: &str) -> bool {
        ["glb"].contains(&extension)
    }

    fn assemble(&mut self, source_path: &Path, target_path: &Path) -> Result<()> {
        println!("Adapting GLB: {:?}", source_path.display());

        let (document, buffers, _images) = import(&source_path)?;

        let mut model_data = self.model_adapter.adapt(&ModelResource {
            document,

            buffers: &buffers,
        })?;

        optimize_model(&mut model_data)?;

        Self::write_model(&target_path, &model_data)?;

        let image_collector = self.image_collector.lock().unwrap();

        let images = image_collector.get_resources();
        Self::write_textures(&target_path, &images)?;

        Ok(())
    }
}
