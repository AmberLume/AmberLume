use crate::world::unique::user_input_unique::UserInputUnique;
use glam::Vec2;
use shipyard::UniqueViewMut;

pub fn pointer_system(mut user_input_unique: UniqueViewMut<UserInputUnique>) {
    if let Some(input) = user_input_unique.input.as_mut() {
        if let Some(position) = input.position(false) {
            user_input_unique.pointer_position = Some(Vec2::new(position.x, position.y));
        }
    }
}
