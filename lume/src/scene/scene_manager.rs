use crate::scene::prepared::test_scene::load_test_scene;
use shipyard::World;

pub struct SceneManager;

impl SceneManager {
    pub fn create() -> Self {
        Self {}
    }

    pub fn set_test_scene(&self, world: &World) {
        load_test_scene(world);
    }
}
