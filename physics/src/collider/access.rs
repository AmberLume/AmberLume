use rapier3d::geometry::SharedShape;
use rapier3d::prelude::ColliderBuilder;
use crate::body::BodyHandle;
use crate::collider::{ColliderDescriptor, ColliderHandle};
use crate::ColliderShape;
use crate::context::PhysicsContext;

impl PhysicsContext {
    pub fn create_collider(
        &mut self,
        parent: BodyHandle,
        descriptor: &ColliderDescriptor,
    ) -> Option<ColliderHandle> {
        let shape = match &descriptor.shape {
            ColliderShape::Box { size } => SharedShape::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0),
            ColliderShape::Capsule { half_height, radius } => SharedShape::capsule_y(*half_height, *radius),
            ColliderShape::ConvexHull { points } => SharedShape::convex_hull(points.as_slice())?,
        };

        let collider = ColliderBuilder::new(shape)
            .translation(descriptor.offset)
            .rotation(descriptor.rotation.to_scaled_axis())
            .density(descriptor.density)
            .friction(descriptor.friction)
            .restitution(descriptor.restitution)
            .build();

        let inner = self.collider_set.insert_with_parent(
            collider,
            parent.inner,
            &mut self.rigid_body_set,
        );

        Some(ColliderHandle { inner })
    }
}
