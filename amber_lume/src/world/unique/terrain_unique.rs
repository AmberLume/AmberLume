use crate::terrain::terrain::Terrain;
use glam::Vec3;
use resource_store::ResourceStore;
use shipyard::Unique;
use std::sync::Arc;

#[derive(Unique)]
pub struct TerrainUnique {
    pub terrain: Terrain,

    pub frozen_observer: Option<Vec3>,
}

impl TerrainUnique {
    pub fn new(resource_store: Arc<ResourceStore>) -> Self {
        Self {
            terrain: Terrain::new(resource_store),

            frozen_observer: None,
        }
    }

    pub fn observer(&mut self, camera: Vec3, freeze_observer: bool) -> Vec3 {
        if freeze_observer {
            return *self.frozen_observer.get_or_insert(camera);
        }

        self.frozen_observer = None;

        camera
    }
}
