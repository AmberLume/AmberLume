use glam::Vec3;
use crate::world::unique::user_input_unique::UserInputUnique;
use shipyard::{IntoIter, UniqueView, View, ViewMut};
use crate::input_handler::hardware_key_codes::HardwareKeyCode;
use crate::world::physics::components::character_physics_component::CharacterPhysicsComponent;
use crate::world::components::user_controllable_component::UserControllableComponent;
use crate::world::unique::render_view_unique::RenderViewUnique;

pub fn user_input_system(
    user_input_unique: UniqueView<UserInputUnique>,
    render_view_unique: UniqueView<RenderViewUnique>,
    user_controllable_component: View<UserControllableComponent>,
    mut character_physics_component: ViewMut<CharacterPhysicsComponent>
) {
    let camera_forward = render_view_unique.resolved_camera.forward();
    let forward_xz = Vec3::new(camera_forward.x, 0.0, camera_forward.z).normalize_or_zero();
    let right_xz = forward_xz.cross(Vec3::Y).normalize_or_zero();

    for (_user_controllable, character_physics) in (&user_controllable_component, &mut character_physics_component).iter() {
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
}
