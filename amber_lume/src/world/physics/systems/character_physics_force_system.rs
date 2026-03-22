use shipyard::{IntoIter, UniqueViewMut, ViewMut};
use tracing::warn;
use crate::world::physics::components::character_physics_component::CharacterPhysicsComponent;
use crate::world::physics::components::physical_body_component::{PhysicalBodyComponent};
use crate::world::physics::physics_world_unique::PhysicsWorldUnique;

pub fn character_physics_force_system(
    mut physical_body: ViewMut<PhysicalBodyComponent>,
    mut character_physics: ViewMut<CharacterPhysicsComponent>,
    mut physics_world_unique: UniqueViewMut<PhysicsWorldUnique>,
) {
    let delta_time = physics_world_unique.handle.fixed_delta_time;

    for _ in 0..physics_world_unique.iterate_count {
        for (character_physics, physical_body) in (&mut character_physics, &mut physical_body).iter() {
            let body_handle = physical_body.rigid_body_handle;

            character_physics.vertical_velocity += character_physics.gravity * delta_time;

            let mut velocity = character_physics.movement_velocity;
            velocity.y = character_physics.vertical_velocity;

            let translation = velocity * delta_time;

            let Some(physical_body_collider) = physical_body.colliders.first() else {
                warn!("Character body must have at least one collider");

                continue;
            };

            let collider_handle = physical_body_collider.handle;

            let effective_movement = physics_world_unique.handle.move_character(
                body_handle,
                collider_handle,
                &translation,
                character_physics.kinematic_character_controller,
            );

            character_physics.is_grounded = effective_movement.grounded;

            if effective_movement.grounded {
                character_physics.vertical_velocity = -0.05;
            } else {
                if character_physics.vertical_velocity > 0.0 && character_physics.vertical_velocity < effective_movement.translation.y {
                    character_physics.vertical_velocity = 0.0;
                }
            }
        }
    }
}
