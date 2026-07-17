use std::collections::HashSet;
use glam::Vec3;
use rapier3d::control::{CharacterAutostep, CharacterLength, KinematicCharacterController};
use rapier3d::parry::query::DefaultQueryDispatcher;
use rapier3d::prelude::QueryFilter;
use crate::body::BodyHandle;
use crate::character::CharacterMovement;
use crate::collider::ColliderHandle;
use crate::context::PhysicsContext;

#[derive(Debug, Clone, Copy)]
pub struct CharacterController {
    pub offset: f32,
    pub autostep_height: f32,
    pub max_slope_climb_angle: f32,
}

impl CharacterController {
    pub fn move_character(
        &self,
        context: &mut PhysicsContext,
        body: BodyHandle,
        collider: ColliderHandle,
        translation: Vec3,
        push_force: f32,
    ) -> CharacterMovement {
        let kinematic = KinematicCharacterController {
            offset: CharacterLength::Relative(self.offset),
            autostep: Some(CharacterAutostep {
                max_height: CharacterLength::Relative(self.autostep_height),
                ..Default::default()
            }),
            max_slope_climb_angle: self.max_slope_climb_angle,
            ..Default::default()
        };

        let query_filter = QueryFilter::default().exclude_rigid_body(body.inner);
        let query_pipeline = context.broad_phase.as_query_pipeline(
            &DefaultQueryDispatcher,
            &context.rigid_body_set,
            &context.collider_set,
            query_filter,
        );

        let rigid_body = &context.rigid_body_set[body.inner];
        debug_assert!(rigid_body.is_kinematic());

        let parry_collider = &context.collider_set[collider.inner];
        let shape = parry_collider.shape();
        let collider_position = rigid_body.position() * parry_collider.position_wrt_parent().unwrap();

        let mut collisions = vec![];

        let effective_movement = kinematic.move_shape(
            context.config.fixed_delta_time,
            &query_pipeline,
            shape,
            &collider_position,
            translation,
            |collision| {
                collisions.push(collision.handle);
            },
        );

        let character_mass = rigid_body.mass();

        if translation != Vec3::ZERO {
            let impulse = translation * character_mass * push_force;
            let mut pushed = HashSet::new();

            for collider_handle in collisions {
                let Some(collider) = context.collider_set.get(collider_handle) else { continue; };
                let Some(parent_handle) = collider.parent() else { continue; };
                if !pushed.insert(parent_handle) { continue; }

                if let Some(object_body) = context.rigid_body_set.get_mut(parent_handle) {
                    if object_body.is_dynamic() {
                        object_body.apply_impulse(impulse, true);
                    }
                }
            }
        }

        let rigid_body = &mut context.rigid_body_set[body.inner];
        let next_translation = rigid_body.translation() + effective_movement.translation;
        
        rigid_body.set_next_kinematic_translation(next_translation);

        CharacterMovement {
            translation: effective_movement.translation,
            grounded: effective_movement.grounded,
        }
    }
}
