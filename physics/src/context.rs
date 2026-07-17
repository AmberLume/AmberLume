use std::collections::HashMap;
use crate::config::PhysicsConfig;
use rapier3d::math::Pose;
use rapier3d::prelude::{
    BroadPhaseBvh, CCDSolver, ColliderSet, ImpulseJointSet, IntegrationParameters, IslandManager,
    MultibodyJointSet, NarrowPhase, PhysicsPipeline, RigidBodyHandle, RigidBodySet,
};

pub struct PhysicsContext {
    pub(crate) config: PhysicsConfig,

    pub(crate) rigid_body_set: RigidBodySet,
    pub(crate) collider_set: ColliderSet,

    pub(crate) integration_parameters: IntegrationParameters,
    pub(crate) physics_pipeline: PhysicsPipeline,
    pub(crate) island_manager: IslandManager,
    pub(crate) broad_phase: BroadPhaseBvh,
    pub(crate) narrow_phase: NarrowPhase,
    pub(crate) impulse_joint_set: ImpulseJointSet,
    pub(crate) multibody_joint_set: MultibodyJointSet,
    pub(crate) ccd_solver: CCDSolver,

    pub(crate) previous_position: HashMap<RigidBodyHandle, Pose>,
    pub(crate) accumulator: f32,
}

impl PhysicsContext {
    pub fn new(config: PhysicsConfig) -> Self {
        let mut integration_parameters = IntegrationParameters::default();
        integration_parameters.dt = config.fixed_delta_time;

        Self {
            config,

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

            previous_position: HashMap::new(),
            accumulator: 0.0,
        }
    }

    pub fn configure(&mut self, config: PhysicsConfig) {
        self.integration_parameters.dt = config.fixed_delta_time;
        self.config = config;
    }

    pub fn advance(&mut self, delta: f32) -> u32 {
        if self.config.paused {
            return 0;
        }

        self.accumulator += delta;

        let mut step_count = 0;
        while self.accumulator > self.config.fixed_delta_time {
            self.accumulator -= self.config.fixed_delta_time;
            step_count += 1;
        }

        step_count
    }

    pub fn step_once(&mut self) {
        for (handle, body) in self.rigid_body_set.iter() {
            if body.is_fixed() {
                continue;
            }

            self.previous_position.insert(handle, *body.position());
        }

        self.physics_pipeline.step(
            self.config.gravity,
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
    }
}
