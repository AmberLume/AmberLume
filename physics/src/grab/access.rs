use crate::body::BodyHandle;
use crate::context::PhysicsContext;
use crate::grab::{GrabConfig, ObjectGrab};
use rapier3d::dynamics::JointAxis;
use rapier3d::prelude::{GenericJointBuilder, JointAxesMask, MotorModel, RigidBodyBuilder};

impl PhysicsContext {
    pub fn create_grab(&mut self, target: BodyHandle, config: &GrabConfig) -> Option<ObjectGrab> {
        let target_body = self.rigid_body_set.get(target.inner)?;

        let local_center_of_mass = target_body.local_center_of_mass();
        let target_mass = target_body.mass();

        let max_force = target_mass * config.linear_acceleration;

        let anchor = RigidBodyBuilder::fixed().build();
        let anchor_handle = self.rigid_body_set.insert(anchor);

        let joint = GenericJointBuilder::new(JointAxesMask::empty())
            .local_anchor2(local_center_of_mass)
            .motor_model(JointAxis::LinX, MotorModel::AccelerationBased)
            .motor_model(JointAxis::LinY, MotorModel::AccelerationBased)
            .motor_model(JointAxis::LinZ, MotorModel::AccelerationBased)
            .motor_max_force(JointAxis::LinX, max_force)
            .motor_max_force(JointAxis::LinY, max_force)
            .motor_max_force(JointAxis::LinZ, max_force)
            .motor_position(
                JointAxis::LinX,
                0.0,
                config.linear_stiffness,
                config.linear_damping,
            )
            .motor_position(
                JointAxis::LinY,
                0.0,
                config.linear_stiffness,
                config.linear_damping,
            )
            .motor_position(
                JointAxis::LinZ,
                0.0,
                config.linear_stiffness,
                config.linear_damping,
            )
            .build();

        let joint = self
            .impulse_joint_set
            .insert(anchor_handle, target.inner, joint, true);

        if let Some(target_body) = self.rigid_body_set.get_mut(target.inner) {
            target_body.set_gravity_scale(0.0, true);
            target_body.enable_ccd(true);
        }

        Some(ObjectGrab {
            target,
            
            anchor: BodyHandle {
                inner: anchor_handle,
            },
            
            joint,
        })
    }
}
