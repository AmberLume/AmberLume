use glam::Vec3;
use shipyard::{IntoIter, UniqueView, UniqueViewMut, ViewMut};
use tracing::warn;
use crate::world::physics::components::character_physics_component::CharacterPhysicsComponent;
use crate::world::physics::components::physical_body_component::PhysicalBodyComponent;
use crate::world::physics::physics_context_unique::PhysicsContextUnique;
use crate::world::unique::render_view_unique::RenderViewUnique;
use crate::world::unique::world_time_unique::WorldTimeUnique;

const GROUND_STICK_VELOCITY: f32 = -0.05;

pub fn physics_step_system(
    world_time_unique: UniqueView<WorldTimeUnique>,
    mut physics_context_unique: UniqueViewMut<PhysicsContextUnique>,
    render_view_unique: UniqueView<RenderViewUnique>,
    mut physical_body: ViewMut<PhysicalBodyComponent>,
    mut character_physics: ViewMut<CharacterPhysicsComponent>,
) {
    physics_context_unique.refresh_config();

    let fixed_delta_time = physics_context_unique.fixed_delta_time;
    let step_count = physics_context_unique.handle.advance(world_time_unique.delta);

    for _ in 0..step_count {
        for (character_physics, physical_body) in (&mut character_physics, &mut physical_body).iter() {
            debug_assert_eq!(
                physical_body.colliders.len(),
                1,
                "character body must have exactly one collider; collider order from rkyv is not stable",
            );

            let Some(physical_body_collider) = physical_body.colliders.first() else {
                warn!("Character body has no collider");
                continue;
            };

            character_physics.vertical_velocity += character_physics.gravity * fixed_delta_time;

            let mut velocity = character_physics.movement_velocity;
            velocity.y = character_physics.vertical_velocity;

            let translation = velocity * fixed_delta_time;

            let effective_movement = character_physics.character_controller.move_character(
                &mut physics_context_unique.handle,
                physical_body.rigid_body_handle,
                physical_body_collider.handle,
                translation,
                character_physics.push_force,
            );

            character_physics.is_grounded = effective_movement.grounded
                && character_physics.vertical_velocity <= 0.0;

            if character_physics.is_grounded {
                character_physics.vertical_velocity = GROUND_STICK_VELOCITY;
            } else if character_physics.vertical_velocity > 0.0
                && effective_movement.translation.y <= 0.0 {
                character_physics.vertical_velocity = 0.0;
            }

            character_physics.movement_velocity = Vec3::ZERO;
        }

        physics_context_unique.handle.step_once();
    }

    physics_context_unique.iterate_count = step_count;

    if step_count > 0 {
        let physics = &mut *physics_context_unique;

        physics.debug_renderer.fill(&physics.handle, render_view_unique.resolved_camera.position, 6.0);
    }
}
