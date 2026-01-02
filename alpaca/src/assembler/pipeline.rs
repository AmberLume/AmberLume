use crate::assembler::utils::{for_each_file, get_extension};
use anyhow::Result;
use std::path::{Path, PathBuf};
use crate::assembler::models::model_pipeline::ModelPipeline;
use crate::assembler::scenes::scene_pipeline::ScenePipeline;
use crate::assembler::shaders::shader_pipeline::ShaderPipeline;

pub struct Pipeline {
    scene_pipeline: ScenePipeline,
    model_pipeline: ModelPipeline,
    shader_pipeline: ShaderPipeline,
}

impl Pipeline {
    pub fn new(
        source_assets: &Path,
    ) -> Result<Self> {
        let scene_pipeline = ScenePipeline::new();
        let model_pipeline = ModelPipeline::new();
        let shader_pipeline = ShaderPipeline::new(&source_assets.join("shaders"))?;

        Ok(Self {
            scene_pipeline,
            model_pipeline,
            shader_pipeline,
        })
    }

    pub fn assemble(&mut self, source_path: &Path, generated_root_path: &Path) -> Result<()> {
        let target_path = &generated_root_path.join("assets");

        self.collect_scenes(&source_path, &target_path.join("scenes"))?;
        self.collect_model(&source_path, &generated_root_path)?;
        self.collect_shaders(&source_path, &target_path.join("shaders"))?;

        Ok(())
    }

    fn collect_scenes(
        &mut self,
        source_path: &Path,
        generated_root_path: &Path,
    ) -> Result<()> {
        let scenes_path = source_path.join("scenes");

        for_each_file(scenes_path.clone(), |path| {
            let file_path = scenes_path.join(path);
            let extension = get_extension(&file_path)?;

            println!("Assembling {}...", file_path.display());

            if self.scene_pipeline.can_assemble(&extension) {
                self.scene_pipeline.collect_scenes(&file_path, &generated_root_path, &path.parent().unwrap())?;
            }

            Ok(())
        })?;

        println!("Used models {:?}...", self.scene_pipeline.used_models);

        Ok(())
    }

    fn collect_model(
        &mut self,
        source_path: &Path,
        generated_root_path: &Path,
    ) -> Result<()> {
        let models_path = source_path.join("models");

        let mut uncollected_models = self.scene_pipeline.used_models.clone();

        for uncollected_model in &self.scene_pipeline.used_models {
            let (file, collection) = uncollected_model.split_once('#').unwrap();

            let file_path = models_path.join(file).with_extension("glb");
            let local_path = PathBuf::new().join("assets").join("models").join(file);

            println!("Assembling {}...", file_path.display());

            if file_path.exists() {
                self.model_pipeline.collect_models(&file_path, &generated_root_path, &local_path, &collection)?;
            } else {
                println!("Linked model file not found! Link: {}", uncollected_model);

                continue;
            }

            let index = uncollected_models.iter().position(|item| item == uncollected_model);
            if let Some(index) = index {
                uncollected_models.remove(index);
            }
        }

        println!("Uncollected models {:?}...", uncollected_models);

        Ok(())
    }

    fn collect_shaders(
        &mut self,
        source_path: &Path,
        generated_root_path: &Path,
    ) -> Result<()> {
        let shaders_path = source_path.join("shaders");

        for_each_file(shaders_path.clone(), |path| {
            let file_path = shaders_path.join(path);
            let extension = get_extension(&file_path)?;

            println!("Assembling {}...", file_path.display());

            if self.shader_pipeline.can_assemble(&extension) {
                self.shader_pipeline.compile(&file_path, &generated_root_path, &path.parent().unwrap())?;
            }

            Ok(())
        })?;

        Ok(())
    }
}
