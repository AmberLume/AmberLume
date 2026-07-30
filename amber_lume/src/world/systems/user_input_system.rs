use glam::Vec3;
use crate::world::unique::user_input_unique::UserInputUnique;
use input::HardwareKeyCode;
use shipyard::{Get, UniqueView, UniqueViewMut, ViewMut};
use crate::world::physics::components::character_physics_component::CharacterPhysicsComponent;
use crate::world::unique::player_control_unique::PlayerControlUnique;
use crate::world::unique::render_view_unique::RenderViewUnique;

pub fn user_input_system(
    mut user_input_unique: UniqueViewMut<UserInputUnique>,
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

    let Some(input) = user_input_unique.input.as_mut() else {
        return;
    };

    let mut linear_velocity = Vec3::ZERO;

    if input.key(HardwareKeyCode::W, true).is_down() {
        linear_velocity += forward_xz;
    }

    if input.key(HardwareKeyCode::S, true).is_down() {
        linear_velocity -= forward_xz;
    }

    if input.key(HardwareKeyCode::D, true).is_down() {
        linear_velocity += right_xz;
    }

    if input.key(HardwareKeyCode::A, true).is_down() {
        linear_velocity -= right_xz;
    }

    if input.key(HardwareKeyCode::Space, true).is_just_pressed() && character_physics.is_grounded {
        character_physics.velocity.y = character_physics.jump_velocity;
    }

    character_physics.movement_velocity = linear_velocity.normalize_or_zero() * character_physics.speed;
}
