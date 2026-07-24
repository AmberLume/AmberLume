use crate::body::BodyHandle;
use crate::context::PhysicsContext;
use glam::Vec3;
use rapier3d::math::Pose;
use rapier3d::parry::query::{DefaultQueryDispatcher, ShapeCastOptions};
use rapier3d::parry::shape::Ball;
use rapier3d::prelude::QueryFilter;

#[derive(Debug, Clone, Copy)]
pub struct SphereSweepHit {
    pub body: BodyHandle,
    pub distance: f32,
}

impl SphereSweepHit {
    pub fn cast(
        context: &PhysicsContext,
        origin: Vec3,
        direction: Vec3,
        radius: f32,
        max_distance: f32,
        exclude: Option<BodyHandle>,
    ) -> Option<SphereSweepHit> {
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

        let (collider_handle, hit) = query_pipeline.cast_shape(
            &Pose::from_translation(origin.into()),
            direction.into(),
            &Ball::new(radius),
            ShapeCastOptions::with_max_time_of_impact(max_distance),
        )?;

        let inner = context.collider_set.get(collider_handle)?.parent()?;

        Some(Self {
            body: BodyHandle { inner },
            distance: hit.time_of_impact,
        })
    }
}
