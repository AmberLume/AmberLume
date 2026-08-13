use std::sync::Arc;
use crate::scene::prepared::test_scene::load_test_scene;
use shipyard::World;
use resource_reader::SceneLoader;

pub struct SceneManager {
    scene_loader: Arc<SceneLoader>,
}

impl SceneManager {
    pub fn create(
        scene_loader: Arc<SceneLoader>,
    ) -> Self {
        Self {
            scene_loader: scene_loader.clone(),
        }
    }

    pub fn set_test_scene(&self, world: &World) {
        load_test_scene(world, &self.scene_loader);
    }
}
