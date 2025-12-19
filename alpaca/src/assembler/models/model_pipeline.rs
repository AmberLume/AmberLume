use crate::assembler::models::aabb_utils::fill_aabbs;
use crate::assembler::models::meshopt_utils::optimize_model;
use crate::assembler::models::model_compiler::{ModelCompiler, ModelResource};
use crate::assembler::resource_compiler::ResourceCompiler;
use crate::assembler::resource_pipeline::ResourcePipeline;
use crate::assembler::utils::{get_name, write_bytes};
use crate::data::adapter::model_adapter::ModelAdapter;
use anyhow::Result;
use gltf::{Node, import};
use std::path::Path;

pub struct ModelPipeline {
    model_compiler: ModelCompiler,
}

impl ModelPipeline {
    pub fn new() -> Result<Self> {
        let model_compiler = ModelCompiler::new()?;

        Ok(Self { model_compiler })
    }

    fn collect_node(node: &Node, meshes: &mut Vec<usize>) {
        if let Some(mesh) = node.mesh() {
            let index = mesh.index();

            meshes.push(index);
        }

        for node in node.children() {
            Self::collect_node(&node, meshes)
        }
    }
}

impl ResourcePipeline for ModelPipeline {
    fn can_assemble(&self, extension: &str) -> bool {
        ["glb"].contains(&extension)
    }

    fn assemble(&self, source_path: &Path, target_path: &Path) -> Result<()> {
        let result_model_path = target_path.with_extension("model");
        let model_name = get_name(&source_path)?;

        println!("Optimizing GLB: {:?}", source_path.display());

        let (document, buffers, _images) = import(&source_path)?;

        let mut scene_meshes = Vec::new();

        for scene in document.scenes() {
            for node in scene.nodes() {
                Self::collect_node(&node, &mut scene_meshes);
            }
        }

        scene_meshes.sort();
        scene_meshes.dedup();

        let mut model_data =
            ModelAdapter::create_from(&document, &buffers, model_name, &scene_meshes)?;

        optimize_model(&mut model_data)?;
        fill_aabbs(&mut model_data);

        let model_resource = ModelResource { data: model_data };

        let model_bytes = self.model_compiler.compile(model_resource)?;

        write_bytes(&result_model_path, &model_bytes)?;

        Ok(())
    }
}
