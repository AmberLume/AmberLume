use crate::assembler::models::mesh_compiler::{MeshCompiler, MeshResource};
use crate::assembler::models::model_compiler::{Mesh, ModelCompiler, ModelResource};
use crate::assembler::resource_compiler::ResourceCompiler;
use crate::assembler::resource_pipeline::ResourcePipeline;
use crate::assembler::utils::write_bytes;
use anyhow::Result;
use gltf::{Node, import};
use std::fs::create_dir_all;
use std::path::Path;

pub struct ModelPipeline {
    mesh_compiler: MeshCompiler,
    model_compiler: ModelCompiler,
}

impl ModelPipeline {
    pub fn new() -> Result<Self> {
        let mesh_compiler = MeshCompiler::new()?;
        let model_compiler = ModelCompiler::new()?;

        Ok(Self {
            mesh_compiler,
            model_compiler,
        })
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
        let result_model_path = target_path.join("manifest");

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

        let document_meshes = document.meshes().collect::<Vec<_>>();

        let mut result_meshes = Vec::new();

        for index in scene_meshes {
            let mesh = &document_meshes[index];
            let mesh_name = mesh.name().expect("Mesh names are required");

            let mut primitive_paths = Vec::new();
            let mut aabbs = Vec::new();

            for primitive in mesh.primitives() {
                let primitive_name = format!("{}#{}.primitive", mesh_name, primitive.index());
                let result_path = target_path.join(&primitive_name);

                create_dir_all(&result_path.parent().unwrap())?;

                println!("Optimizing {:?}", &primitive_name);

                let (indices, vertices) =
                    self.mesh_compiler.optimize_geometry(&primitive, &buffers)?;

                let aabb = self.mesh_compiler.calculate_aabb(&vertices);

                let mesh_resource = MeshResource { indices, vertices };
                let result = self.mesh_compiler.compile(mesh_resource)?;

                write_bytes(&result_path, &result)?;

                aabbs.push(aabb);

                primitive_paths.push(primitive_name);
            }

            let aabb = self.mesh_compiler.calculate_global_aabb(&aabbs);

            let mesh = Mesh {
                name: mesh_name.to_owned(),
                primitives: primitive_paths,
                bounds: aabb,
            };

            result_meshes.push(mesh);
        }

        let model_aabb = self.mesh_compiler.calculate_global_aabb(
            &result_meshes
                .iter()
                .map(|m| m.bounds)
                .collect::<Vec<[f32; 6]>>(),
        );

        let model_resource = ModelResource {
            meshes: result_meshes,
            bounds: model_aabb,
        };

        let model_bytes = self.model_compiler.compile(model_resource)?;

        write_bytes(&result_model_path, &model_bytes)?;

        Ok(())
    }
}
