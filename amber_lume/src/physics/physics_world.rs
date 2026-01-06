use std::collections::HashMap;
use glam::{Quat, Vec3};
use nalgebra::Vector3;
use rapier3d::control::{EffectiveCharacterMovement, KinematicCharacterController};
use rapier3d::parry::query::DefaultQueryDispatcher;
use rapier3d::prelude::{BroadPhaseBvh, CCDSolver, ColliderBuilder, ColliderHandle, ColliderSet, ImpulseJointSet, IntegrationParameters, IslandManager, Isometry, MultibodyJointSet, NarrowPhase, PhysicsPipeline, QueryFilter, RigidBodyBuilder, RigidBodyHandle, RigidBodySet};
use crate::physics::body_type::BodyType;
use crate::physics::collider_shape::ColliderShape;
use crate::physics::utils::{euler_from_quat, shape_from, vector3_from_vec3};

pub struct PhysicsWorld {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,

    integration_parameters: IntegrationParameters,

    physics_pipeline: PhysicsPipeline,

    island_manager: IslandManager,

    broad_phase: BroadPhaseBvh,
    narrow_phase: NarrowPhase,

    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,

    ccd_solver: CCDSolver,

    gravity: Vector3<f32>,

    previous_position: HashMap<RigidBodyHandle, Isometry<f32>>,

    pub fixed_delta_time: f32,
    accumulator: f32,
}

impl PhysicsWorld {
    pub const GRAVITY: Vector3<f32> = Vector3::new(0.0, -9.81, 0.0);

    pub fn create() -> Self {
        let fixed_delta_time = 1.0 / 60.0;

        let mut integration_parameters = IntegrationParameters::default();
        integration_parameters.dt = fixed_delta_time;

        Self {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),

            integration_parameters,

            physics_pipeline: PhysicsPipeline::new(),

            island_manager: IslandManager::new(),

            broad_phase: BroadPhaseBvh::new(),
            narrow_phase: NarrowPhase::new(),

            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),

            ccd_solver: CCDSolver::new(),

            gravity: Self::GRAVITY,

            previous_position: HashMap::default(),

            fixed_delta_time,
            accumulator: 0.0,
        }
    }

    pub fn set_step_delta(&mut self, delta_time: f32) {
        self.integration_parameters.dt = delta_time;
        self.fixed_delta_time = delta_time;
    }

    pub fn step(&mut self, delta: f32) -> u32 {
        self.accumulator += delta;

        let mut step_count = 0;
        while self.accumulator > self.fixed_delta_time {
            step_count += 1;

            for (handle, body) in self.rigid_body_set.iter() {
                let position = *body.position();

                self.previous_position.insert(handle, position);
            }

            self.physics_pipeline.step(
                &self.gravity,
                &self.integration_parameters,
                &mut self.island_manager,
                &mut self.broad_phase,
                &mut self.narrow_phase,
                &mut self.rigid_body_set,
                &mut self.collider_set,
                &mut self.impulse_joint_set,
                &mut self.multibody_joint_set,
                &mut self.ccd_solver,
                &(),
                &(),
            );

            self.accumulator -= self.fixed_delta_time;
        }

        step_count
    }

    pub fn move_character(
        &mut self,
        handle: RigidBodyHandle,
        collider_handle: ColliderHandle,
        translation: &Vec3,
        controller: KinematicCharacterController,
    ) -> EffectiveCharacterMovement {
        let translation = vector3_from_vec3(translation);

        let query_filter = QueryFilter::default()
            .exclude_collider(collider_handle);
        let query_pipeline = self.broad_phase.as_query_pipeline(
            &DefaultQueryDispatcher,
            &self.rigid_body_set,
            &self.collider_set,
            query_filter,
        );

        let body = self.rigid_body_set.get(handle).unwrap();
        let collider = self.collider_set.get(collider_handle).unwrap();
        let shape = collider.shape();

        let mut collisions = vec![];

        let effective_movement = controller.move_shape(
            self.fixed_delta_time,
            &query_pipeline,
            shape,
            &body.position(),
            translation,
            |collision| {
                collisions.push(collision.handle);
            },
        );

        for collider_handle in collisions {
            if let Some(collider) = self.collider_set.get(collider_handle) {
                if let Some(parent_handle) = collider.parent() {
                    if let Some(body) = self.rigid_body_set.get_mut(parent_handle) {
                        if body.is_dynamic() {
                            let push_direction = translation.normalize();
                            let impulse = push_direction * 0.5;
                            body.apply_impulse(impulse, true);
                        }
                    }
                }
            }
        }


        let rigid_body = self.rigid_body_set.get_mut(handle).unwrap();

        let translation = rigid_body.translation() + effective_movement.translation;

        rigid_body.set_next_kinematic_translation(translation);

        effective_movement
    }

    pub fn create_parent(
        &mut self,
        body_type: &BodyType,
        position: &Vec3,
        rotation: &Quat,
    ) -> RigidBodyHandle {
        let position = vector3_from_vec3(position);
        let rotation = euler_from_quat(rotation);

        let rigid_body_builder = match body_type {
            BodyType::Static => RigidBodyBuilder::fixed().lock_rotations(),
            BodyType::Kinematic => RigidBodyBuilder::kinematic_position_based().lock_rotations(),
            BodyType::Dynamic => RigidBodyBuilder::dynamic(),
        };

        let rigid_body = rigid_body_builder
            .translation(position)
            .rotation(rotation)
            .build();

        self.rigid_body_set.insert(rigid_body)
    }

    pub fn add_collider(
        &mut self,
        parent_handle: RigidBodyHandle,
        position: &Vec3,
        rotation: &Quat,
        collider_shape: &ColliderShape,
    ) -> ColliderHandle {
        let position = vector3_from_vec3(&position);
        let rotation = euler_from_quat(&rotation);

        let shape = shape_from(&collider_shape);

        let collider = ColliderBuilder::new(shape)
            .translation(position)
            .rotation(rotation)
            .friction(0.99)
            .build();

        self.collider_set.insert_with_parent(collider, parent_handle, &mut self.rigid_body_set)
    }

    pub fn remove(&mut self, handle: RigidBodyHandle) {
        self.rigid_body_set.remove(
            handle,
            &mut self.island_manager,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            true,
        );
        self.previous_position.remove(&handle);
    }

    pub fn get_interpolated_position_rotation(&self, handle: RigidBodyHandle) -> (Vec3, Quat) {
        let rigid_body = self.rigid_body_set.get(handle).expect("Entity missing rigid body");
        let current_position = rigid_body.position();
        let alpha = self.interpolation_factor();

        let current_translation = current_position.translation;
        let current_rotation = current_position.rotation;

        let (translation, rotation) = if let Some(previous_position) = self.previous_position.get(&handle) {
            let previous_translation = previous_position.translation;
            let previous_rotation = previous_position.rotation;

            let translation = previous_translation.vector.lerp(&current_translation.vector, alpha);
            let rotation = previous_rotation.slerp(&current_rotation, alpha);

            let translation = Vec3::new(translation.x, translation.y, translation.z);
            let rotation = Quat::from_xyzw(rotation.i, rotation.j, rotation.k, rotation.w);

            (translation, rotation)
        } else {
            let translation = Vec3::new(current_translation.x, current_translation.y, current_translation.z);
            let rotation = Quat::from_xyzw(current_rotation.i, current_rotation.j, current_rotation.k, current_rotation.w);

            (translation, rotation)
        };

        (translation, rotation)
    }

    pub fn interpolation_factor(&self) -> f32 {
        self.accumulator / self.fixed_delta_time
    }
}
