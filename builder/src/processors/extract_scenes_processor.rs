use std::sync::Arc;
use anyhow::Result;
use gltf::Scene;
use log::{info, warn};
use serde::Deserialize;
use serde_json::from_str;
use crate::build_task::{BuildTask, CollectSceneTask, ExtractScenesTask};
use crate::dispatcher::Dispatcher;
use crate::gltf_file::GltfFile;
use crate::paths::Paths;
use crate::processors::processor::Processor;

#[derive(Deserialize, Debug, PartialEq)]
struct SceneExtras {
    pub scene_name: String,
}

pub struct ExtractScenesProcessor;

impl ExtractScenesProcessor {
    pub fn create() -> Self {
        Self {
            
        }
    }

    fn extract_scene_extras(&self, paths: &Paths, scene: &Scene) -> Option<SceneExtras> {
        if let Some(raw_extras) = scene.extras() {
            if let Ok(extras) = from_str::<SceneExtras>(raw_extras.get()) {
                Some(extras)
            } else {
                warn!("Error while deserializing scene extras to SceneExtras! Path: {}, extras: {:?}", paths.relative.display(), raw_extras);

                None
            }
        } else {
            warn!("Scene does not have a scene extras! Path: {}, name: {:?}", paths.relative.display(), scene.name().unwrap());

            None
        }
    }
}

impl Processor<ExtractScenesTask> for ExtractScenesProcessor {
    fn process(&self, dispatcher: Arc<Dispatcher>, task: &ExtractScenesTask) -> Result<()> {
        info!("Parsing GLTF {}", task.paths.relative.display());

        let path = task.paths.source_file();

        let gltf_file = Arc::new(GltfFile::create(&path)?);

        let document = gltf_file.get_document()?;

        document.scenes().for_each(|scene| {
            let dispatcher = dispatcher.clone();

            let scene_extras = self.extract_scene_extras(&task.paths, &scene);

            if let Some(scene_extras) = scene_extras {
                dispatcher.dispatch(BuildTask::CollectScene(CollectSceneTask {
                    name: scene_extras.scene_name,

                    scene_index: scene.index(),
                    gltf_file: gltf_file.clone(),

                    paths: task.paths.clone(),
                }))
            }
        });

        Ok(())
    }
}
