use crate::physics::physics_world::PhysicsWorld;
use rapier3d::prelude::RigidBodyHandle;

pub struct BodyParameters;

impl BodyParameters {
    pub fn is_dynamic(physics_world: &PhysicsWorld, body: RigidBodyHandle) -> bool {
        physics_world
            .rigid_body_set
            .get(body)
            .map(|body| body.is_dynamic())
            .unwrap_or(false)
    }
}
