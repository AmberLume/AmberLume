use rapier3d::dynamics::RigidBodyBuilder;
use crate::body::{BodyDescriptor, BodyHandle, BodyType};
use crate::context::PhysicsContext;

impl PhysicsContext {
    pub fn create_body(&mut self, descriptor: &BodyDescriptor) -> BodyHandle {
        let builder = match descriptor.body_type {
            BodyType::Static => RigidBodyBuilder::fixed().lock_rotations(),
            BodyType::Kinematic => RigidBodyBuilder::kinematic_position_based().lock_rotations(),
            BodyType::Dynamic => RigidBodyBuilder::dynamic(),
        };

        let rigid_body = builder
            .translation(descriptor.position)
            .rotation(descriptor.rotation.to_scaled_axis())
            .build();

        BodyHandle {
            inner: self.rigid_body_set.insert(rigid_body),
        }
    }
}
