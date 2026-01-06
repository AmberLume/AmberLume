use shipyard::Unique;
use crate::physics::physics_world::PhysicsWorld;

#[derive(Unique)]
pub struct PhysicsWorldUnique {
    pub handle: PhysicsWorld,
    
    pub iterate_count: u32,
}

impl PhysicsWorldUnique {
    pub fn new() -> Self {
        let physics_world = PhysicsWorld::create();
        
        Self {
            handle: physics_world,
            
            iterate_count: 0,
        }
    }
}
