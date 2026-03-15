use std::sync::Arc;
use crate::resources::index::resource_index::ResourceIndex;
use anyhow::Result;
use rkyv::{access, deserialize};
use rkyv::rancor::Error;
use builder::data::scene_data::{ArchivedSceneData, SceneData};

pub struct SceneLoader {
    resource_index: Arc<ResourceIndex>,
}

impl SceneLoader {
    pub fn create(
        resource_index: Arc<ResourceIndex>,
    ) -> Self {
        Self {
            resource_index,
        }
    }
    
    pub fn load(&self, name: &str) -> Result<SceneData> {
        let name = &format!("scenes/{}.scene", name);
        
        let scene_bytes = self.resource_index.get_resource(name)?;
        let archived = access::<ArchivedSceneData, Error>(&scene_bytes)?;

        let scene_data = deserialize::<SceneData, Error>(archived)?;
        
        Ok(scene_data)
    }
}
