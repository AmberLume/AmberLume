use glam::Vec3;
use rapier3d::dynamics::JointAxis;
use rapier3d::prelude::{GenericJointBuilder, ImpulseJointHandle, JointAxesMask, MotorModel, RigidBodyBuilder, RigidBodyHandle};
use crate::physics::physics_world::PhysicsWorld;
use crate::world::components::grab_component::GrabParams;

#[derive(Debug, Clone, Copy)]
pub struct ObjectGrab {
    target_handle: RigidBodyHandle,

    anchor_handle: RigidBodyHandle,
    joint_handle: ImpulseJointHandle,
}

impl ObjectGrab {
    pub fn grab(
        physics_world: &mut PhysicsWorld,
        target_handle: RigidBodyHandle,
        params: &GrabParams,
    ) -> Option<Self> {
        let (local_center_of_mass, mass) = physics_world
            .rigid_body_set
            .get(target_handle)
            .map(|target_body| (target_body.local_center_of_mass(), target_body.mass()))
            .unwrap_or((Vec3::ZERO, 1.0));

        let max_force = mass * params.grab_acceleration;

        let anchor = RigidBodyBuilder::fixed().build();
        let anchor_handle = physics_world.rigid_body_set.insert(anchor);

        let joint = GenericJointBuilder::new(JointAxesMask::empty())
            .local_anchor2(local_center_of_mass)
            .motor_model(JointAxis::LinX, MotorModel::AccelerationBased)
            .motor_model(JointAxis::LinY, MotorModel::AccelerationBased)
            .motor_model(JointAxis::LinZ, MotorModel::AccelerationBased)
            .motor_max_force(JointAxis::LinX, max_force)
            .motor_max_force(JointAxis::LinY, max_force)
            .motor_max_force(JointAxis::LinZ, max_force)
            .motor_position(JointAxis::LinX, 0.0, params.linear_stiffness, params.linear_damping)
            .motor_position(JointAxis::LinY, 0.0, params.linear_stiffness, params.linear_damping)
            .motor_position(JointAxis::LinZ, 0.0, params.linear_stiffness, params.linear_damping)
            .build();
        let joint_handle = physics_world
            .impulse_joint_set
            .insert(anchor_handle, target_handle, joint, true);

        if let Some(target_body) = physics_world.rigid_body_set.get_mut(target_handle) {
            target_body.set_gravity_scale(0.0, true);
            target_body.enable_ccd(true);
        }

        Some(Self {
            target_handle,

            anchor_handle,
            joint_handle,
        })
    }

    pub fn move_anchor(
        &self,
        physics_world: &mut PhysicsWorld,
        target_position: Vec3,
        params: GrabParams,
    ) {
        if let Some(joint) = physics_world.impulse_joint_set.get_mut(self.joint_handle, true) {
            joint.data.set_motor(JointAxis::LinX, target_position.x, 0.0, params.linear_stiffness, params.linear_damping);
            joint.data.set_motor(JointAxis::LinY, target_position.y, 0.0, params.linear_stiffness, params.linear_damping);
            joint.data.set_motor(JointAxis::LinZ, target_position.z, 0.0, params.linear_stiffness, params.linear_damping);
        }
    }

    pub fn release(&self, physics_world: &mut PhysicsWorld) {
        if let Some(target_body) = physics_world.rigid_body_set.get_mut(self.target_handle) {
            target_body.set_gravity_scale(1.0, true);
            target_body.enable_ccd(false);
        }

        physics_world.impulse_joint_set.remove(self.joint_handle, true);
        physics_world.remove(self.anchor_handle);
    }
}
