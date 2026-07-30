use std::collections::HashSet;
use glam::Vec3;
use rapier3d::geometry::{ContactManifold, Shape};
use rapier3d::math::Pose;
use rapier3d::parry::bounding_volume::BoundingVolume;
use rapier3d::parry::query::{DefaultQueryDispatcher, PersistentQueryDispatcher, ShapeCastOptions};
use rapier3d::prelude::{QueryFilter, QueryPipeline};
use crate::body::BodyHandle;
use crate::character::{CharacterMoveRequest, CharacterMovement, FloorContact, SlideMovement, SweepHit};
use crate::collider::ColliderHandle;
use crate::context::PhysicsContext;

#[derive(Debug, Clone, Copy)]
pub struct CharacterController {
    pub offset: f32,
    pub autostep_height: f32,
    pub max_slope_climb_angle: f32,
}

impl CharacterController {
    const SLIDE_ITERATIONS: u32 = 4;
    const COINCIDENT_PLANE_COSINE: f32 = 0.99;
    const FLOOR_CONTACT_MARGIN: f32 = 0.05;
    const MINIMUM_MOTION: f32 = 1.0e-5;

    pub fn move_character(
        &self,
        context: &mut PhysicsContext,
        body: BodyHandle,
        collider: ColliderHandle,
        request: &CharacterMoveRequest,
    ) -> CharacterMovement {
        let delta_time = context.config.fixed_delta_time;

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
        let start = rigid_body.position() * parry_collider.position_wrt_parent().unwrap();

        let up_extent = shape.compute_local_aabb().extents().y;
        let skin = self.offset * up_extent;
        let step_height = self.autostep_height * up_extent;
        let contact_range = skin + Self::FLOOR_CONTACT_MARGIN;
        let step_down_range = step_height + skin;

        let mut collisions = vec![];

        let mut translation = self.depenetrate(&query_pipeline, shape, &start);

        let standing_floor = self.floor_at(
            &query_pipeline,
            shape,
            &(Pose::from_translation(translation) * start),
            contact_range,
            step_down_range,
        );

        let mut horizontal = Vec3::new(
            request.input_velocity.x,
            0.0,
            request.input_velocity.z,
        ) * delta_time;

        if request.is_grounded {
            if let Some(floor) = &standing_floor {
                let along = horizontal - floor.normal * horizontal.dot(floor.normal);

                if along.length() > Self::MINIMUM_MOTION {
                    horizontal = along.normalize() * horizontal.length();
                }
            }
        }

        let horizontal_movement = self.slide(
            &query_pipeline,
            shape,
            &(Pose::from_translation(translation) * start),
            horizontal,
            Vec3::ZERO,
            skin,
            &mut collisions,
        );

        translation += match request.is_grounded {
            true => Vec3::new(
                horizontal_movement.translation.x,
                horizontal_movement.translation.y.min(0.0),
                horizontal_movement.translation.z,
            ),
            false => horizontal_movement.translation,
        };

        if request.is_grounded && horizontal_movement.blocked {
            let remaining = horizontal - horizontal_movement.translation;

            if remaining.length() > Self::MINIMUM_MOTION {
                let step = self.step_up(
                    &query_pipeline,
                    shape,
                    &(Pose::from_translation(translation) * start),
                    remaining,
                    skin,
                    step_height,
                    &mut collisions,
                );

                if let Some(step) = step {
                    translation += step;
                }
            }
        }

        let advanced = Pose::from_translation(translation) * start;
        let adheres_to_floor = request.is_grounded && request.velocity.y <= 0.0;

        let adherence_floor = adheres_to_floor
            .then(|| self.floor_at(&query_pipeline, shape, &advanced, contact_range, step_down_range))
            .flatten();

        let velocity = match adherence_floor {
            Some(floor) => {
                let adherence = self.slide(
                    &query_pipeline,
                    shape,
                    &advanced,
                    Vec3::Y * (skin - floor.distance),
                    Vec3::ZERO,
                    skin,
                    &mut collisions,
                );

                translation += Vec3::Y * adherence.translation.y;

                Vec3::ZERO
            }
            None => {
                let fall = self.slide(
                    &query_pipeline,
                    shape,
                    &advanced,
                    request.velocity * delta_time,
                    request.velocity,
                    skin,
                    &mut collisions,
                );

                translation += fall.translation;

                fall.velocity
            }
        };

        let settled = Pose::from_translation(translation) * start;
        let grounded = self
            .contact_floor(&query_pipeline, shape, &settled, contact_range)
            .is_some();

        let character_mass = rigid_body.mass();

        if horizontal != Vec3::ZERO {
            let impulse = horizontal * character_mass * request.push_force;
            let mut pushed = HashSet::new();

            for collider_handle in collisions {
                let Some(collider) = context.collider_set.get(collider_handle.inner) else { continue; };
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
        let next_translation = rigid_body.translation() + translation;

        rigid_body.set_next_kinematic_translation(next_translation);

        CharacterMovement {
            translation,
            velocity,
            grounded,
        }
    }

    fn slide(
        &self,
        query_pipeline: &QueryPipeline,
        shape: &dyn Shape,
        start: &Pose,
        motion: Vec3,
        motion_velocity: Vec3,
        skin: f32,
        collisions: &mut Vec<ColliderHandle>,
    ) -> SlideMovement {
        let minimum_ground_cosine = self.max_slope_climb_angle.cos();

        let mut translation = Vec3::ZERO;
        let mut remaining = motion;
        let mut velocity = motion_velocity;
        let mut blocked = false;
        let mut clipped: [Vec3; Self::SLIDE_ITERATIONS as usize] = [Vec3::ZERO; Self::SLIDE_ITERATIONS as usize];
        let mut clipped_count = 0;

        for _ in 0..Self::SLIDE_ITERATIONS {
            let distance = remaining.length();

            if distance <= Self::MINIMUM_MOTION {
                break;
            }

            let direction = remaining / distance;
            let position = Pose::from_translation(translation) * *start;

            let Some(hit) = self.cast(query_pipeline, shape, &position, direction, distance) else {
                translation += remaining;
                break;
            };

            collisions.push(hit.collider);

            let advance = (hit.distance - skin).clamp(0.0, distance);

            translation += direction * advance;
            remaining -= direction * advance;

            let surface = hit.normal;

            if surface.dot(Vec3::Y) < minimum_ground_cosine {
                blocked = true;
            }

            remaining -= surface * remaining.dot(surface);

            let repeated = clipped[..clipped_count]
                .iter()
                .any(|applied| applied.dot(surface) > Self::COINCIDENT_PLANE_COSINE);

            if !repeated {
                clipped[clipped_count] = surface;
                clipped_count += 1;

                let into_surface = velocity.dot(surface);

                if into_surface < 0.0 {
                    velocity -= surface * into_surface;
                }
            }
        }

        SlideMovement { translation, velocity, blocked }
    }

    fn step_up(
        &self,
        query_pipeline: &QueryPipeline,
        shape: &dyn Shape,
        start: &Pose,
        remaining: Vec3,
        skin: f32,
        step_height: f32,
        collisions: &mut Vec<ColliderHandle>,
    ) -> Option<Vec3> {
        let rise = match self.cast(query_pipeline, shape, start, Vec3::Y, step_height) {
            Some(hit) => (hit.distance - skin).clamp(0.0, step_height),
            None => step_height,
        };

        if rise <= Self::MINIMUM_MOTION {
            return None;
        }

        let raised = Pose::from_translation(Vec3::Y * rise) * *start;
        let forward = self.slide(
            query_pipeline,
            shape,
            &raised,
            remaining,
            Vec3::ZERO,
            skin,
            collisions,
        );

        if forward.translation.length() <= Self::MINIMUM_MOTION {
            return None;
        }

        let advanced = Pose::from_translation(forward.translation) * raised;
        let landing = self.cast_floor(query_pipeline, shape, &advanced, rise + skin)?;
        let drop = (landing.distance - skin).max(0.0);

        Some(Vec3::Y * (rise - drop) + forward.translation)
    }

    fn floor_at(
        &self,
        query_pipeline: &QueryPipeline,
        shape: &dyn Shape,
        position: &Pose,
        contact_range: f32,
        step_down_range: f32,
    ) -> Option<FloorContact> {
        self.contact_floor(query_pipeline, shape, position, contact_range)
            .or_else(|| self.cast_floor(query_pipeline, shape, position, step_down_range))
    }

    fn contact_floor(
        &self,
        query_pipeline: &QueryPipeline,
        shape: &dyn Shape,
        position: &Pose,
        range: f32,
    ) -> Option<FloorContact> {
        let minimum_ground_cosine = self.max_slope_climb_angle.cos();

        let probe = shape.compute_aabb(position).loosened(range);
        let mut manifolds: Vec<ContactManifold> = vec![];
        let mut closest: Option<FloorContact> = None;

        for (_, collider) in query_pipeline.intersect_aabb_conservative(probe) {
            manifolds.clear();

            let relative_position = position.inv_mul(collider.position());

            let _ = DefaultQueryDispatcher.contact_manifolds(
                &relative_position,
                shape,
                collider.shape(),
                range,
                &mut manifolds,
                &mut None,
            );

            for manifold in &manifolds {
                let normal = -(position.rotation * manifold.local_n1);

                if normal.dot(Vec3::Y) < minimum_ground_cosine {
                    continue;
                }

                for contact in &manifold.points {
                    if contact.dist > range {
                        continue;
                    }

                    let closer = match &closest {
                        Some(floor) => contact.dist < floor.distance,
                        None => true,
                    };

                    if closer {
                        closest = Some(FloorContact {
                            distance: contact.dist,
                            normal,
                        });
                    }
                }
            }
        }

        closest
    }

    fn cast_floor(
        &self,
        query_pipeline: &QueryPipeline,
        shape: &dyn Shape,
        position: &Pose,
        range: f32,
    ) -> Option<FloorContact> {
        let hit = self.cast(query_pipeline, shape, position, Vec3::NEG_Y, range)?;

        if hit.normal.dot(Vec3::Y) < self.max_slope_climb_angle.cos() {
            return None;
        }

        Some(FloorContact {
            distance: hit.distance,
            normal: hit.normal,
        })
    }

    fn cast(
        &self,
        query_pipeline: &QueryPipeline,
        shape: &dyn Shape,
        position: &Pose,
        direction: Vec3,
        distance: f32,
    ) -> Option<SweepHit> {
        let (collider, hit) = query_pipeline.cast_shape(
            position,
            direction,
            shape,
            ShapeCastOptions {
                target_distance: 0.0,
                stop_at_penetration: true,
                max_time_of_impact: distance,
                compute_impact_geometry_on_penetration: true,
            },
        )?;

        Some(SweepHit {
            collider: ColliderHandle { inner: collider },
            distance: hit.time_of_impact,
            normal: hit.normal1,
        })
    }

    fn depenetrate(
        &self,
        query_pipeline: &QueryPipeline,
        shape: &dyn Shape,
        position: &Pose,
    ) -> Vec3 {
        let probe = shape.compute_aabb(position);
        let mut manifolds: Vec<ContactManifold> = vec![];
        let mut deepest: Option<FloorContact> = None;

        for (_, collider) in query_pipeline.intersect_aabb_conservative(probe) {
            manifolds.clear();

            let relative_position = position.inv_mul(collider.position());

            let _ = DefaultQueryDispatcher.contact_manifolds(
                &relative_position,
                shape,
                collider.shape(),
                0.0,
                &mut manifolds,
                &mut None,
            );

            for manifold in &manifolds {
                let normal = -(position.rotation * manifold.local_n1);

                for contact in &manifold.points {
                    if contact.dist >= 0.0 {
                        continue;
                    }

                    let deeper = match &deepest {
                        Some(contact_point) => contact.dist < contact_point.distance,
                        None => true,
                    };

                    if deeper {
                        deepest = Some(FloorContact {
                            distance: contact.dist,
                            normal,
                        });
                    }
                }
            }
        }

        match deepest {
            Some(contact) => contact.normal * -contact.distance,
            None => Vec3::ZERO,
        }
    }
}
