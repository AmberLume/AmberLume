use std::sync::Arc;
use anyhow::Result;
use tracing::{info, warn};
use crate::build_task::{BuildTask, ExtractAssetsTask, RouteTarget, ShaderTask};
use crate::dispatcher::Dispatcher;
use crate::processors::processor::Processor;

pub struct RouteTargetProcessor;

impl RouteTargetProcessor {
    pub fn create() -> Self { Self }
}

impl Processor<RouteTarget> for RouteTargetProcessor {
    fn process(&self, dispatcher: Arc<Dispatcher>, task: &RouteTarget) -> Result<()> {
        let extension = &task.build_target.extension;

        info!("Routing... {:?}", task.build_target.relative_full());

        match extension.as_str() {
            "vert" | "frag" | "comp" => {
                dispatcher.dispatch(BuildTask::CompileShader(ShaderTask {
                    build_target: task.build_target.clone(),
                }))
            },
            "gltf" => {
                dispatcher.dispatch(BuildTask::ExtractScenes(ExtractAssetsTask {
                    build_target: task.build_target.clone(),
                }))
            },
            "png" | "jpg" => {
                // Skipping images
            },
            "glsl" => {
                // Skipping dependency file
            },
            _ => {
                warn!("Unsupported file extension: {}", extension);
            } 
        }

        Ok(())
    }
}
