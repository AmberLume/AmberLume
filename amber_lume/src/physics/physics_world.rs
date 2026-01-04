use std::collections::HashMap;
use glam::{EulerRot, Quat, Vec3};
use nalgebra::Vector3;
use rapier3d::prelude::{AngVector, BroadPhaseBvh, CCDSolver, ColliderBuilder, ColliderSet, ImpulseJointSet, IntegrationParameters, IslandManager, Isometry, MultibodyJointSet, NarrowPhase, PhysicsPipeline, RigidBodyBuilder, RigidBodyHandle, RigidBodySet, SharedShape};

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

    fixed_delta_time: f32,
    accumulator: f32,
}

impl PhysicsWorld {
    const GRAVITY: Vector3<f32> = Vector3::new(0.0, -9.81, 0.0);

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

    pub fn step(&mut self, delta: f32) {
        self.accumulator += delta;

        while self.accumulator > self.fixed_delta_time {
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
    }

    pub fn add_static(
        &mut self,
        entity_position: &Vec3,
        entity_rotation: &Quat,
        position: &Vec3,
        rotation: &Quat,
        shape: SharedShape,
    ) -> RigidBodyHandle {
        let entity_position = Vector3::new(entity_position.x, entity_position.y, entity_position.z);
        let entity_rotation = Self::euler_from_quat(entity_rotation);

        let entity_rigid_body = RigidBodyBuilder::fixed()
            .translation(entity_position)
            .rotation(entity_rotation)
            .build();
        let entity_handle = self.rigid_body_set.insert(entity_rigid_body);

        self.put_collider_to(entity_handle, &position, &rotation, shape);

        entity_handle
    }

    pub fn add_kinematic(
        &mut self,
        entity_position: &Vec3,
        entity_rotation: &Quat,
        position: &Vec3,
        rotation: &Quat,
        shape: SharedShape,
    ) -> RigidBodyHandle {
        let entity_position = Vector3::new(entity_position.x, entity_position.y, entity_position.z);
        let entity_rotation = Self::euler_from_quat(entity_rotation);

        let entity_rigid_body = RigidBodyBuilder::kinematic_position_based()
            .translation(entity_position)
            .rotation(entity_rotation)
            .build();
        let entity_handle = self.rigid_body_set.insert(entity_rigid_body);

        self.put_collider_to(entity_handle, &position, &rotation, shape);

        entity_handle
    }

    pub fn add_dynamic(
        &mut self,
        entity_position: &Vec3,
        entity_rotation: &Quat,
        position: &Vec3,
        rotation: &Quat,
        shape: SharedShape,
    ) -> RigidBodyHandle {
        let entity_position = Self::vector3_from_vec3(&entity_position);
        let entity_rotation = Self::euler_from_quat(&entity_rotation);

        let entity_rigid_body = RigidBodyBuilder::dynamic()
            .translation(entity_position)
            .rotation(entity_rotation)
            .build();
        let entity_handle = self.rigid_body_set.insert(entity_rigid_body);

        self.put_collider_to(entity_handle, &position, &rotation, shape);

        entity_handle
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

    fn put_collider_to(
        self: &mut Self,
        parent: RigidBodyHandle,
        position: &Vec3,
        rotation: &Quat,
        shape: SharedShape,
    ) {
        let position = Self::vector3_from_vec3(&position);
        let rotation = Self::euler_from_quat(&rotation);

        let collider = ColliderBuilder::new(shape)
            .translation(position)
            .rotation(rotation)
            .build();
        self.collider_set.insert_with_parent(collider, parent, &mut self.rigid_body_set);
    }

    fn vector3_from_vec3(vec3: &Vec3) -> Vector3<f32> {
        Vector3::new(vec3.x, vec3.y, vec3.z)
    }

    fn euler_from_quat(quat: &Quat) -> AngVector<f32> {
        let euler_rotation = quat.to_euler(EulerRot::XYZ);

        AngVector::new(euler_rotation.0, euler_rotation.1, euler_rotation.2)
    }
}
