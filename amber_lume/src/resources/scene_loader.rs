use std::sync::Arc;
use crate::resources::alpaca_resource_reader::AlpacaResourceReader;
use anyhow::Result;
use rkyv::{access, deserialize};
use rkyv::rancor::Error;
use crate::data::resource_handle::SceneResource;
use crate::data::scene_data::{ArchivedSceneData, SceneData};

pub struct SceneLoader {
    alpaca_resource_reader: Arc<AlpacaResourceReader>,
}

impl SceneLoader {
    pub fn create(
        alpaca_resource_reader: Arc<AlpacaResourceReader>,
    ) -> Self {
        Self {
            alpaca_resource_reader,
        }
    }
    
    pub fn load(&self, scene: SceneResource) -> Result<SceneData> {
        let scene_bytes = self.alpaca_resource_reader.get_resource(scene.key())?;
        let archived = access::<ArchivedSceneData, Error>(&scene_bytes)?;

        let scene_data = deserialize::<SceneData, Error>(archived)?;
        
        Ok(scene_data)
    }
}
