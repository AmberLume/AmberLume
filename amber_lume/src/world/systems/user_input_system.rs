use glam::Vec3;
use crate::world::unique::user_input_unique::UserInputUnique;
use shipyard::{IntoIter, UniqueView, View, ViewMut};
use crate::input_handler::keycodes::Keycode;
use crate::world::physics::components::character_physics_component::CharacterPhysicsComponent;
use crate::world::components::user_controllable_component::UserControllableComponent;

pub fn user_input_system(
    user_input_unique: UniqueView<UserInputUnique>,
    user_controllable_component: View<UserControllableComponent>,
    mut character_physics_component: ViewMut<CharacterPhysicsComponent>
) {
    for (_user_controllable, character_physics) in (&user_controllable_component, &mut character_physics_component).iter() {
        let mut linear_velocity = Vec3::new(0.0, 0.0, 0.0);

        if user_input_unique.state.is_down(Keycode::ArrowUp) {
            linear_velocity.z -= 1.0;
        }

        if user_input_unique.state.is_down(Keycode::ArrowLeft) {
            linear_velocity.x -= 1.0;
        }

        if user_input_unique.state.is_down(Keycode::ArrowDown) {
            linear_velocity.z += 1.0;
        }

        if user_input_unique.state.is_down(Keycode::ArrowRight) {
            linear_velocity.x += 1.0;
        }

        if user_input_unique.state.is_down(Keycode::C) && character_physics.is_grounded {
            character_physics.vertical_velocity = 10.0;
        }

        character_physics.movement_velocity = linear_velocity.normalize_or_zero() * character_physics.speed;
    }
}
