use glam::Vec3;
use rapier3d::parry::query::{DefaultQueryDispatcher, Ray};
use rapier3d::prelude::{QueryFilter, RigidBodyHandle};
use crate::physics::physics_world::PhysicsWorld;

#[derive(Debug, Clone, Copy)]
pub struct RayHit {
    pub body: RigidBodyHandle,
    pub point: Vec3,
    pub distance: f32,
    pub normal: Vec3,
}

impl RayHit {
    pub fn cast(
        physics_world: &PhysicsWorld,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
        exclude: Option<RigidBodyHandle>,
    ) -> Option<RayHit> {
        let direction = direction.normalize();

        let mut query_filter = QueryFilter::default();
        if let Some(handle) = exclude {
            query_filter = query_filter.exclude_rigid_body(handle);
        }

        let query_pipeline = physics_world.broad_phase.as_query_pipeline(
            &DefaultQueryDispatcher,
            &physics_world.rigid_body_set,
            &physics_world.collider_set,
            query_filter,
        );

        let ray = Ray::new(origin.into(), direction.into());
        let (collider_handle, intersection) = query_pipeline.cast_ray_and_get_normal(&ray, max_distance, true)?;

        let body = physics_world.collider_set.get(collider_handle)?.parent()?;

        Some(Self {
            body,
            point: origin + direction * intersection.time_of_impact,
            distance: intersection.time_of_impact,
            normal: intersection.normal,
        })
    }
}
