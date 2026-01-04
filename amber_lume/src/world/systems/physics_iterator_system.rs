use glam::{Quat, Vec3};
use rapier3d::prelude::{RigidBodyHandle, SharedShape};
use shipyard::{IntoIter, UniqueView, UniqueViewMut, ViewMut};
use crate::physics::physics_world::PhysicsWorld;
use crate::world::components::physical_body_component::{BodyCollider, BodyColliderType, BodyColliderShape, PhysicalBodyComponent};
use crate::world::components::position_component::PositionComponent;
use crate::world::components::rotation_component::RotationComponent;
use crate::world::unique::physics_world_unique::PhysicsWorldUnique;
use crate::world::unique::world_time_unique::WorldTimeUnique;

pub fn physics_iterator_system(
    mut physical_body: ViewMut<PhysicalBodyComponent>,
    mut position: ViewMut<PositionComponent>,
    mut rotation: ViewMut<RotationComponent>,
    world_time_unique: UniqueView<WorldTimeUnique>,
    mut physics_world_unique: UniqueViewMut<PhysicsWorldUnique>,
) {
    let delta_time = world_time_unique.delta;
    physics_world_unique.handle.step(delta_time);

    for (position, rotation, physical_body) in (&mut position, &mut rotation, &mut physical_body).iter() {
        for collider in &mut physical_body.colliders {
            let handle = if let Some(handle) = collider.handle {
                handle
            } else {
                let handle = add_to_world(&mut physics_world_unique.handle, &position.position, &rotation.rotation, &collider);

                collider.handle = Some(handle);

                handle
            };

            let (body_position, body_rotation) = physics_world_unique.handle.get_interpolated_position_rotation(handle);

            position.position = body_position;
            rotation.rotation = body_rotation;
        }
    }
}

fn add_to_world(
    physics_world: &mut PhysicsWorld,
    position: &Vec3,
    rotation: &Quat,
    collider: &BodyCollider,
) -> RigidBodyHandle {
    let shape = match collider.shape {
        BodyColliderShape::Box { size } => {
            SharedShape::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0)
        },
    };

    match collider.collider_type {
        BodyColliderType::Static => {
            physics_world.add_static(
                &position,
                &rotation,
                &collider.position,
                &collider.rotation,
                shape,
            )
        },
        BodyColliderType::Kinematic => physics_world.add_kinematic(
            &position,
            &rotation,
            &collider.position,
            &collider.rotation,
            shape,
        ),
        BodyColliderType::Dynamic => physics_world.add_dynamic(
            &position,
            &rotation,
            &collider.position,
            &collider.rotation,
            shape,
        ),
    }
}
