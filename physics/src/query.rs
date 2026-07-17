use crate::body::BodyHandle;
use crate::context::PhysicsContext;
use glam::Vec3;
use rapier3d::parry::query::{DefaultQueryDispatcher, Ray};
use rapier3d::prelude::QueryFilter;

#[derive(Debug, Clone, Copy)]
pub struct RayHit {
    pub body: BodyHandle,
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
}

impl RayHit {
    pub fn cast(
        context: &PhysicsContext,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
        exclude: Option<BodyHandle>,
    ) -> Option<RayHit> {
        let direction = direction.normalize();

        let mut query_filter = QueryFilter::default();
        if let Some(handle) = exclude {
            query_filter = query_filter.exclude_rigid_body(handle.inner);
        }

        let query_pipeline = context.broad_phase.as_query_pipeline(
            &DefaultQueryDispatcher,
            &context.rigid_body_set,
            &context.collider_set,
            query_filter,
        );

        let ray = Ray::new(origin.into(), direction.into());
        let (collider_handle, intersection) = query_pipeline
            .cast_ray_and_get_normal(&ray, max_distance, true)?;

        let inner = context.collider_set.get(collider_handle)?.parent()?;

        Some(Self {
            body: BodyHandle { inner },
            point: origin + direction * intersection.time_of_impact,
            normal: intersection.normal,
            distance: intersection.time_of_impact,
        })
    }
}
