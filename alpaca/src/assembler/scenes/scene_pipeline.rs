use crate::assembler::adapter::adapter::ResourceAdapter;
use crate::assembler::utils::{get_name, write_bytes};
use anyhow::Result;
use gltf::import;
use rkyv::rancor::Error;
use rkyv::to_bytes;
use std::fs::create_dir_all;
use std::path::Path;
use crate::assembler::adapter::scene_adapter::{SceneAdapter, SceneResource};
use crate::data::common::scene_data::SceneData;

pub struct ScenePipeline {
    scene_adapter: SceneAdapter,

    pub used_models: Vec<String>,
}

impl ScenePipeline {
    pub fn new() -> Self {
        let scene_adapter = SceneAdapter::create();

        Self {
            scene_adapter,

            used_models: Vec::new(),
        }
    }

    pub fn can_assemble(&self, extension: &str) -> bool {
        ["glb"].contains(&extension)
    }

    pub fn collect_scenes(&mut self, source_path: &Path, generated_root_path: &Path, local_path: &Path) -> Result<()> {
        let name = get_name(source_path)?;
        let local_path = &local_path.with_extension("");

        let result_path = generated_root_path.join(local_path).join(&name);

        println!("Collecting scenes from: {:?}", source_path.display());

        let (document, _buffers, _images) = import(&source_path)?;

        let scenes_data = self.scene_adapter.adapt(&SceneResource {
            document,
        })?;

        for scene_data in scenes_data {
            println!("Collected scene '{:?}'.", scene_data.name);

            for scene_node in &scene_data.nodes {
                println!("Collected node '{}' (position = {:?}, rotation = {:?}, scale = {:?}).", scene_node.name, scene_node.transform, scene_node.rotation, scene_node.scale);

                if !self.used_models.contains(&scene_node.asset_key) {
                    self.used_models.push(scene_node.asset_key.clone());
                }
            }

            Self::write_scene(&result_path, &scene_data)?;
        }

        Ok(())
    }

    fn write_scene(target_path: &Path, scene_data: &SceneData) -> Result<()> {
        create_dir_all(&target_path)?;

        let result_scene_path = target_path.join(&scene_data.name).with_extension("scene");

        let scene_bytes = to_bytes::<Error>(scene_data)?.into_vec();

        write_bytes(&result_scene_path, &scene_bytes)?;

        Ok(())
    }
}
