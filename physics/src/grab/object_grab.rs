use glam::Vec3;
use rapier3d::dynamics::JointAxis;
use rapier3d::prelude::ImpulseJointHandle;
use crate::body::BodyHandle;
use crate::context::PhysicsContext;
use crate::grab::GrabConfig;

#[derive(Debug, Clone, Copy)]
pub struct ObjectGrab {
    pub(crate) target: BodyHandle,
    pub(crate) anchor: BodyHandle,
    pub(crate) joint: ImpulseJointHandle,
}

impl ObjectGrab {
    pub fn move_anchor(&self, context: &mut PhysicsContext, target_position: Vec3, config: &GrabConfig) {
        if let Some(joint) = context.impulse_joint_set.get_mut(self.joint, true) {
            joint.data.set_motor(
                JointAxis::LinX,
                target_position.x,
                0.0,
                config.linear_stiffness,
                config.linear_damping,
            );
            joint.data.set_motor(
                JointAxis::LinY,
                target_position.y,
                0.0,
                config.linear_stiffness,
                config.linear_damping,
            );
            joint.data.set_motor(
                JointAxis::LinZ,
                target_position.z,
                0.0,
                config.linear_stiffness,
                config.linear_damping,
            );
        }
    }

    pub fn release(self, context: &mut PhysicsContext) {
        if let Some(target_body) = context.rigid_body_set.get_mut(self.target.inner) {
            target_body.set_gravity_scale(1.0, true);
            target_body.enable_ccd(false);
        }

        context.impulse_joint_set.remove(self.joint, true);

        self.anchor.remove(context);
    }
}
