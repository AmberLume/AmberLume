use glam::Vec3;
use rapier3d::control::{CharacterAutostep, CharacterLength, KinematicCharacterController};
use shipyard::Component;
use crate::physics::physics_world::PhysicsWorld;

#[derive(Component, Debug)]
pub struct CharacterPhysicsComponent {
    pub kinematic_character_controller: KinematicCharacterController,

    pub speed: f32,
    pub gravity: f32,
    pub push_force: f32,
    pub jump_velocity: f32,

    pub movement_velocity: Vec3,
    pub vertical_velocity: f32,
    pub is_grounded: bool,
}

impl CharacterPhysicsComponent {
    pub fn create(
        offset: f32,
        auto_step_height: f32,
        max_angle: f32,
        speed: f32,
        push_force: f32,
        jump_velocity: f32,
    ) -> Self {
        let kinematic_character_controller = KinematicCharacterController {
            offset: CharacterLength::Relative(offset),
            autostep: Some(CharacterAutostep {
                max_height: CharacterLength::Relative(auto_step_height),
                ..Default::default()
            }),
            max_slope_climb_angle: max_angle.to_radians(),
            ..Default::default()
        };

        Self {
            kinematic_character_controller,

            speed,
            gravity: PhysicsWorld::GRAVITY.y,
            push_force,
            jump_velocity,

            movement_velocity: Vec3::ZERO,
            vertical_velocity: 0.0,
            is_grounded: false,
        }
    }
}
