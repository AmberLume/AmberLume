use math::{Quat, Vec3};
use rapier3d::prelude::RigidBodyHandle;
use crate::context::PhysicsContext;
use crate::rapier_math::to_math::ToMath;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BodyHandle {
    pub(crate) inner: RigidBodyHandle,
}

impl BodyHandle {
    pub fn position(&self, context: &PhysicsContext) -> Vec3 {
        context.rigid_body_set[self.inner].translation().to_math()
    }

    pub fn rotation(&self, context: &PhysicsContext) -> Quat {
        context.rigid_body_set[self.inner].rotation().to_math()
    }

    pub fn mass(&self, context: &PhysicsContext) -> f32 {
        context.rigid_body_set[self.inner].mass()
    }

    pub fn is_dynamic(&self, context: &PhysicsContext) -> bool {
        context.rigid_body_set[self.inner].is_dynamic()
    }

    pub fn interpolated_position_rotation(&self, context: &PhysicsContext) -> (Vec3, Quat) {
        let position = context.rigid_body_set[self.inner].position();
        let current_translation = position.translation.to_math();
        let current_rotation = position.rotation.to_math();

        if !context.config.interpolation {
            return (current_translation, current_rotation);
        }

        match context.previous_position.get(&self.inner) {
            Some(previous) => {
                let alpha = context.accumulator / context.config.fixed_delta_time;

                (
                    previous.translation.to_math().lerp(current_translation, alpha),
                    previous.rotation.to_math().slerp(current_rotation, alpha),
                )
            }
            None => (current_translation, current_rotation),
        }
    }

    pub fn set_gravity_scale(&self, context: &mut PhysicsContext, scale: f32) {
        context.rigid_body_set[self.inner].set_gravity_scale(scale, true);
    }

    pub fn enable_ccd(&self, context: &mut PhysicsContext, enabled: bool) {
        context.rigid_body_set[self.inner].enable_ccd(enabled);
    }

    pub fn remove(&self, context: &mut PhysicsContext) {
        context.rigid_body_set.remove(
            self.inner,
            &mut context.island_manager,
            &mut context.collider_set,
            &mut context.impulse_joint_set,
            &mut context.multibody_joint_set,
            true,
        );

        context.previous_position.remove(&self.inner);
    }
}
