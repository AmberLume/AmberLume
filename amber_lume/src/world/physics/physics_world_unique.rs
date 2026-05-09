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
        fixed_delta_time: f32,
    ) -> Self {
        let physics_world = PhysicsWorld::create(settings, fixed_delta_time);

        Self {
            handle: physics_world,

            iterate_count: 0,
        }
    }
}
