use glam::Vec3;
use crate::world::unique::user_input_unique::UserInputUnique;
use shipyard::{Get, UniqueView, ViewMut};
use crate::input_handler::hardware_key_codes::HardwareKeyCode;
use crate::world::physics::components::character_physics_component::CharacterPhysicsComponent;
use crate::world::unique::player_control_unique::PlayerControlUnique;
use crate::world::unique::render_view_unique::RenderViewUnique;

pub fn user_input_system(
    user_input_unique: UniqueView<UserInputUnique>,
    render_view_unique: UniqueView<RenderViewUnique>,
    player_control_unique: UniqueView<PlayerControlUnique>,
    mut character_physics_component: ViewMut<CharacterPhysicsComponent>,
) {
    let Some(controlled) = player_control_unique.controlled else {
        return;
    };

    let Ok(mut character_physics) = (&mut character_physics_component).get(controlled) else {
        return;
    };

    let camera_forward = render_view_unique.resolved_camera.forward();
    let forward_xz = Vec3::new(camera_forward.x, 0.0, camera_forward.z).normalize_or_zero();
    let right_xz = forward_xz.cross(Vec3::Y).normalize_or_zero();

    let mut linear_velocity = Vec3::ZERO;

    if user_input_unique.input_frame.is_down(HardwareKeyCode::W) {
        linear_velocity += forward_xz;
    }

    if user_input_unique.input_frame.is_down(HardwareKeyCode::S) {
        linear_velocity -= forward_xz;
    }

    if user_input_unique.input_frame.is_down(HardwareKeyCode::D) {
        linear_velocity += right_xz;
    }

    if user_input_unique.input_frame.is_down(HardwareKeyCode::A) {
        linear_velocity -= right_xz;
    }

    if user_input_unique.input_frame.just_pressed(HardwareKeyCode::Space) && character_physics.is_grounded {
        character_physics.vertical_velocity = character_physics.jump_velocity;
    }

    character_physics.movement_velocity = linear_velocity.normalize_or_zero() * character_physics.speed;
}
