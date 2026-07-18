use crate::body::BodyHandle;
use crate::context::PhysicsContext;
use glam::{Quat, Vec3};
use rapier3d::dynamics::JointAxis;
use rapier3d::prelude::ImpulseJointHandle;

#[derive(Debug, Clone, Copy)]
pub struct ObjectGrab {
    pub(crate) target: BodyHandle,

    pub(crate) anchor: BodyHandle,
    pub(crate) joint: ImpulseJointHandle,

    pub(crate) angular_anchor: BodyHandle,
    pub(crate) angular_joint: ImpulseJointHandle,

    pub(crate) relative_rotation: Quat,
}

impl ObjectGrab {
    pub fn rotate(&mut self, delta_rotation: Quat) {
        self.relative_rotation = (delta_rotation * self.relative_rotation).normalize();
    }

    pub fn move_anchor(
        &self,
        context: &mut PhysicsContext,
        target_position: Vec3,
        camera_rotation: Quat,
    ) {
        if let Some(joint) = context.impulse_joint_set.get_mut(self.joint, true) {
            joint.data.motors[JointAxis::LinX as usize].target_pos = target_position.x;
            joint.data.motors[JointAxis::LinY as usize].target_pos = target_position.y;
            joint.data.motors[JointAxis::LinZ as usize].target_pos = target_position.z;
        }

        let Some(anchor) = context.rigid_body_set.get_mut(self.angular_anchor.inner) else {
            return;
        };

        let mut target_rotation = camera_rotation * self.relative_rotation;
        if target_rotation.dot(*anchor.rotation()) < 0.0 {
            target_rotation = -target_rotation;
        }

        anchor.set_next_kinematic_rotation(target_rotation);
    }

    pub fn release(self, context: &mut PhysicsContext) {
        if let Some(target_body) = context.rigid_body_set.get_mut(self.target.inner) {
            target_body.set_gravity_scale(1.0, true);
            target_body.enable_ccd(false);
        }

        context.impulse_joint_set.remove(self.joint, true);
        context.impulse_joint_set.remove(self.angular_joint, true);

        self.anchor.remove(context);
        self.angular_anchor.remove(context);
    }
}
