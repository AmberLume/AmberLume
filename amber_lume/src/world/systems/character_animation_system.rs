use shipyard::{IntoIter, View, ViewMut};
use crate::world::components::animation_parameters_component::{AnimationFlagParameter, AnimationFloatParameter, AnimationParametersComponent};
use crate::world::physics::components::character_physics_component::CharacterPhysicsComponent;

pub fn character_animation_system(
    character_physics: View<CharacterPhysicsComponent>,
    mut animation_parameters_components: ViewMut<AnimationParametersComponent>,
) {
    for (physics, parameters) in (&character_physics, &mut animation_parameters_components).iter() {
        parameters.floats[AnimationFloatParameter::Speed as usize] = physics.movement_velocity.length();
        parameters.floats[AnimationFloatParameter::VerticalSpeed as usize] = physics.velocity.y;
        parameters.flags[AnimationFlagParameter::Grounded as usize] = physics.is_grounded;
    }
}
