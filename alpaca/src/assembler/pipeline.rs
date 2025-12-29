use crate::assembler::models::model_pipeline::ModelPipeline;
use crate::assembler::resource_pipeline::ResourcePipeline;
use crate::assembler::shaders::shader_pipeline::ShaderPipeline;
use crate::assembler::utils::{for_each_file, get_extension};
use anyhow::Result;
use std::path::Path;

pub struct Pipeline {
    pipelines: Vec<Box<dyn ResourcePipeline>>,
}

impl Pipeline {
    pub fn new(
        source_assets: &Path,
    ) -> Result<Self> {
        let shader_pipeline = ShaderPipeline::new(&source_assets)?;
        let model_pipeline = ModelPipeline::new();

        let pipelines: Vec<Box<dyn ResourcePipeline>> =
            vec![Box::new(shader_pipeline), Box::new(model_pipeline)];

        Ok(Self { pipelines })
    }

    pub fn assemble(&mut self, source_path: &Path, generated_root_path: &Path) -> Result<()> {
        for_each_file(source_path, |path| {
            let file_path = source_path.join(path);
            let extension = get_extension(&file_path)?;

            println!("Assembling {}...", file_path.display());

            let mut assembled = false;

            for pipeline in &mut self.pipelines {
                if pipeline.can_assemble(&extension) {
                    pipeline.assemble(&file_path, &generated_root_path, &path.parent().unwrap())?;

                    assembled = true;
                }
            }

            if !assembled {
                println!("Unassembled resource: {}", source_path.display());
            }

            Ok(())
        })?;

        Ok(())
    }
}
