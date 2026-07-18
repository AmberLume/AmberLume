use glam::Quat;
use rapier3d::dynamics::JointAxis;
use rapier3d::prelude::{GenericJointBuilder, JointAxesMask, MotorModel, RigidBodyBuilder};
use crate::body::BodyHandle;
use crate::context::PhysicsContext;
use crate::grab::{GrabConfig, ObjectGrab};

impl PhysicsContext {
    pub fn create_grab(
        &mut self,
        target: BodyHandle,
        config: &GrabConfig,
        camera_rotation: Quat,
    ) -> Option<ObjectGrab> {
        let (local_center_of_mass, target_mass, object_rotation, angular_inertia) = {
            let target_body = self.rigid_body_set.get(target.inner)?;

            (
                target_body.local_center_of_mass(),
                target_body.mass(),
                *target_body.rotation(),
                target_body.mass_properties().local_mprops.principal_inertia().max_element(),
            )
        };

        let max_force = target_mass * config.linear_acceleration;
        let max_torque = angular_inertia * config.angular_acceleration;
        let relative_rotation = camera_rotation.inverse() * object_rotation;

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
            .motor_position(JointAxis::LinX, 0.0, config.linear_stiffness, config.linear_damping)
            .motor_position(JointAxis::LinY, 0.0, config.linear_stiffness, config.linear_damping)
            .motor_position(JointAxis::LinZ, 0.0, config.linear_stiffness, config.linear_damping)
            .build();

        let joint = self
            .impulse_joint_set
            .insert(anchor_handle, target.inner, joint, true);

        let angular_anchor = RigidBodyBuilder::kinematic_position_based()
            .rotation(object_rotation.to_scaled_axis())
            .build();
        let angular_anchor_handle = self.rigid_body_set.insert(angular_anchor);

        let angular_joint = GenericJointBuilder::new(JointAxesMask::empty())
            .motor_model(JointAxis::AngX, MotorModel::AccelerationBased)
            .motor_model(JointAxis::AngY, MotorModel::AccelerationBased)
            .motor_model(JointAxis::AngZ, MotorModel::AccelerationBased)
            .motor_max_force(JointAxis::AngX, max_torque)
            .motor_max_force(JointAxis::AngY, max_torque)
            .motor_max_force(JointAxis::AngZ, max_torque)
            .motor_position(JointAxis::AngX, 0.0, config.angular_stiffness, config.angular_damping)
            .motor_position(JointAxis::AngY, 0.0, config.angular_stiffness, config.angular_damping)
            .motor_position(JointAxis::AngZ, 0.0, config.angular_stiffness, config.angular_damping)
            .build();

        let angular_joint = self
            .impulse_joint_set
            .insert(angular_anchor_handle, target.inner, angular_joint, true);

        if let Some(target_body) = self.rigid_body_set.get_mut(target.inner) {
            target_body.set_gravity_scale(0.0, true);
            target_body.enable_ccd(true);
        }

        Some(ObjectGrab {
            target,

            anchor: BodyHandle { inner: anchor_handle },
            joint,

            angular_anchor: BodyHandle { inner: angular_anchor_handle },
            angular_joint,

            relative_rotation,
        })
    }
}
