use glam::Vec3;
use physics::CharacterController;
use shipyard::Component;
use crate::world::physics::physics_context_unique::PhysicsContextUnique;

#[derive(Component, Debug)]
pub struct CharacterPhysicsComponent {
    pub character_controller: CharacterController,

    pub speed: f32,
    pub gravity: f32,
    pub push_force: f32,
    pub jump_velocity: f32,

    pub movement_velocity: Vec3,
    pub velocity: Vec3,
    
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
        let character_controller = CharacterController {
            offset,
            autostep_height: auto_step_height,
            max_slope_climb_angle: max_angle.to_radians(),
        };

        Self {
            character_controller,

            speed,
            gravity: PhysicsContextUnique::GRAVITY.y,
            push_force,
            jump_velocity,

            movement_velocity: Vec3::ZERO,
            velocity: Vec3::ZERO,
            
            is_grounded: false,
        }
    }
}
