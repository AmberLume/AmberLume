use crate::resource_reader::ResourceReader;
use anyhow::Result;
use resource_data::resource_handle::SceneResource;
use resource_data::scene_data::{ArchivedSceneData, SceneData};
use rkyv::rancor::Error;
use rkyv::{access, deserialize};
use std::sync::Arc;

pub struct SceneLoader {
    resource_reader: Arc<dyn ResourceReader>,
}

impl SceneLoader {
    pub fn create(resource_reader: Arc<dyn ResourceReader>) -> Self {
        Self { resource_reader }
    }

    pub fn load(&self, scene: SceneResource) -> Result<SceneData> {
        let scene_bytes = self.resource_reader.get_resource(scene.key())?;
        let archived = access::<ArchivedSceneData, Error>(&scene_bytes)?;

        let scene_data = deserialize::<SceneData, Error>(archived)?;

        Ok(scene_data)
    }
}
