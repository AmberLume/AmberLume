use std::sync::Arc;
use arc_swap::ArcSwap;
use shipyard::Unique;
use crate::physics::physics_world::PhysicsWorld;
use crate::settings::settings::EngineSettings;

#[derive(Unique)]
pub struct PhysicsWorldUnique {
    pub handle: PhysicsWorld,
    
    pub iterate_count: u32,
}

impl PhysicsWorldUnique {
    pub fn new(
        settings: Arc<ArcSwap<EngineSettings>>,
    ) -> Self {
        let physics_world = PhysicsWorld::create(settings);

        Self {
            handle: physics_world,
            
            iterate_count: 0,
        }
    }
}
