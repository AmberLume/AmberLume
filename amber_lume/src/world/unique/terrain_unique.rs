use crate::terrain::terrain::Terrain;
use glam::Vec3;
use shipyard::Unique;

#[derive(Unique)]
pub struct TerrainUnique {
    pub terrain: Terrain,

    pub frozen_observer: Option<Vec3>,
}

impl TerrainUnique {
    pub fn new(terrain: Terrain) -> Self {
        Self {
            terrain,

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
