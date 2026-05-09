use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use arc_swap::ArcSwap;
use glam::{Quat, Vec3};
use rapier3d::control::{EffectiveCharacterMovement, KinematicCharacterController};
use rapier3d::math::Pose;
use rapier3d::parry::query::DefaultQueryDispatcher;
use rapier3d::prelude::{BroadPhaseBvh, CCDSolver, ColliderBuilder, ColliderHandle, ColliderSet, DebugRenderMode, DebugRenderPipeline, DebugRenderStyle, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet, NarrowPhase, PhysicsPipeline, QueryFilter, RigidBodyBuilder, RigidBodyHandle, RigidBodySet};
use tracing::warn;
use crate::physics::body_type::BodyType;
use crate::physics::physics_debug_render::{PhysicsDebugLine, PhysicsDebugRender};
use crate::physics::utils::shared_shape_from;
use crate::settings::settings::EngineSettings;
use crate::world::physics::data::ColliderData;

pub struct PhysicsWorld {
    physics_debug_render: PhysicsDebugRender,
    debug_render_pipeline: DebugRenderPipeline,

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

    gravity: Vec3,

    previous_position: HashMap<RigidBodyHandle, Pose>,

    settings: Arc<ArcSwap<EngineSettings>>,

    pub fixed_delta_time: f32,
    accumulator: f32,
}

impl PhysicsWorld {
    pub const GRAVITY: Vec3 = Vec3::new(0.0, -9.81, 0.0);

    pub fn create(
        settings: Arc<ArcSwap<EngineSettings>>,
    ) -> Self {
        let fixed_delta_time = 1.0 / 60.0;

        let mut integration_parameters = IntegrationParameters::default();
        integration_parameters.dt = fixed_delta_time;

        let physics_debug_render = PhysicsDebugRender::new();
        let debug_render_pipeline = DebugRenderPipeline::new(
            DebugRenderStyle::default(),
            DebugRenderMode::COLLIDER_SHAPES,
        );

        Self {
            physics_debug_render,
            debug_render_pipeline,

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

            settings,

            fixed_delta_time,
            accumulator: 0.0,
        }
    }

    pub fn advance_frame(&mut self, delta: f32) -> u32 {
        self.accumulator = self.accumulator + delta;

        let mut step_count = 0;
        while self.accumulator > self.fixed_delta_time {
            step_count += 1;
            self.accumulator -= self.fixed_delta_time;
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
            self.gravity,
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

    pub fn update_debug_lines(&mut self, position: &Vec3, radius: f32) {
        if !self.settings.load().debug.collider_rendering_enabled.get() {
            return;
        } 
        
        self.physics_debug_render.reset();
        self.physics_debug_render.set_position_radius(position, radius);

        self.debug_render_pipeline.render(
            &mut self.physics_debug_render,
            &self.rigid_body_set,
            &self.collider_set,
            &self.impulse_joint_set,
            &self.multibody_joint_set,
            &self.narrow_phase,
        );
    }

    pub fn get_debug_lines(&self) -> &[PhysicsDebugLine] {
        &self.physics_debug_render.lines
    }

    pub fn move_character(
        &mut self,
        handle: RigidBodyHandle,
        collider_handle: ColliderHandle,
        translation: Vec3,
        push_force: f32,
        controller: KinematicCharacterController,
    ) -> EffectiveCharacterMovement {
        let query_filter = QueryFilter::default()
            .exclude_rigid_body(handle);
        let query_pipeline = self.broad_phase.as_query_pipeline(
            &DefaultQueryDispatcher,
            &self.rigid_body_set,
            &self.collider_set,
            query_filter,
        );

        let body = self.rigid_body_set.get(handle).unwrap();
        debug_assert!(
            body.is_kinematic(),
            "move_character requires a kinematic rigid body; set_next_kinematic_translation is a no-op otherwise",
        );
        let collider = self.collider_set.get(collider_handle).unwrap();
        let shape = collider.shape();

        let collider_position = body.position() * collider.position_wrt_parent().unwrap();

        let mut collisions = vec![];

        let effective_movement = controller.move_shape(
            self.fixed_delta_time,
            &query_pipeline,
            shape,
            &collider_position,
            translation,
            |collision| {
                collisions.push(collision.handle);
            },
        );

        let character_mass = body.mass();

        if translation != Vec3::ZERO {
            let impulse = translation * character_mass * push_force;
            let mut pushed = HashSet::new();

            for collider_handle in collisions {
                let Some(collider) = self.collider_set.get(collider_handle) else { continue; };
                let Some(parent_handle) = collider.parent() else { continue; };
                if !pushed.insert(parent_handle) { continue; }

                if let Some(object_body) = self.rigid_body_set.get_mut(parent_handle) {
                    if object_body.is_dynamic() {
                        object_body.apply_impulse(impulse, true);
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
        position: Vec3,
        rotation: &Quat,
    ) -> RigidBodyHandle {
        let rigid_body_builder = match body_type {
            BodyType::Static => RigidBodyBuilder::fixed().lock_rotations(),
            BodyType::Kinematic => RigidBodyBuilder::kinematic_position_based().lock_rotations(),
            BodyType::Dynamic => RigidBodyBuilder::dynamic(),
        };

        let rigid_body = rigid_body_builder
            .translation(position)
            .rotation(rotation.to_scaled_axis())
            .build();

        self.rigid_body_set.insert(rigid_body)
    }

    pub fn add_collider(
        &mut self,
        parent_handle: RigidBodyHandle,
        scale: Vec3,
        collider: &ColliderData,
    ) -> Option<ColliderHandle> {
        if let Some(shape) = shared_shape_from(scale, &collider) {
            let collider = ColliderBuilder::new(shape)
                .translation(collider.translation)
                .rotation(collider.rotation.to_scaled_axis())
                .density(collider.density)
                .friction(collider.friction)
                .restitution(collider.restitution)
                .build();

            let handle = self.collider_set.insert_with_parent(collider, parent_handle, &mut self.rigid_body_set);

            Some(handle)
        } else {
            warn!("Failed to create shape for collider");

            None
        }
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

        let (translation, rotation) = if self.settings.load().debug.physics_interpolation.get() {
            if let Some(previous_position) = self.previous_position.get(&handle) {
                let previous_translation = previous_position.translation;
                let previous_rotation = previous_position.rotation;

                let translation = previous_translation.lerp(current_translation, alpha);
                let rotation = previous_rotation.slerp(current_rotation, alpha);

                (translation, rotation)
            } else {
                (current_translation, current_rotation)
            }
        } else {
            (current_translation, current_rotation)
        };

        (translation, rotation)
    }

    pub fn interpolation_factor(&self) -> f32 {
        self.accumulator / self.fixed_delta_time
    }
}
