use std::sync::Arc;
use anyhow::Result;
use tracing::{info, warn};
use crate::build_task::{BuildTask, ExtractAssetsTask, SeedFileTask, ShaderTask};
use crate::dispatcher::Dispatcher;
use crate::processors::processor::Processor;

pub struct SeedFileProcessor;

impl SeedFileProcessor {
    pub fn create() -> Self {
        Self {
            
        }
    }
}

impl Processor<SeedFileTask> for SeedFileProcessor {
    fn process(&self, dispatcher: Arc<Dispatcher>, task: &SeedFileTask) -> Result<()> {
        let extension = task.paths.extension.as_str();

        info!("Routing file {}", task.paths.relative.display());

        match extension {
            "vert" | "frag" | "comp" => {
                dispatcher.dispatch(BuildTask::CompileShader(ShaderTask {
                    paths: task.paths.clone(),
                }))
            },
            "gltf" => {
                dispatcher.dispatch(BuildTask::ExtractScenes(ExtractAssetsTask {
                    paths: task.paths.clone(),
                }))
            },
            "png" | "jpg" => {
                dispatcher.dispatch(BuildTask::ExtractScenes(ExtractAssetsTask {
                    paths: task.paths.clone(),
                }))
            },
            "glsl" => {
                info!("Skipping dependency file: {}", task.paths.relative.display());
            },
            _ => {
                warn!("Unsupported file extension: {}", extension);
            } 
        }

        Ok(())
    }
}
