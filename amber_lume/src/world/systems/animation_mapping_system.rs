use shipyard::{IntoIter, View, ViewMut};
use crate::animation::animation_state::AnimationState;
use crate::animation::animation_states::humanoid_animation_state::HumanoidAnimationState;
use crate::world::components::animation_component::AnimationComponent;
use crate::world::physics::components::character_physics_component::CharacterPhysicsComponent;

pub fn humanoid_animation_system(
    character_physics: View<CharacterPhysicsComponent>,
    mut animations: ViewMut<AnimationComponent<HumanoidAnimationState>>,
) {
    for (physics, anim) in (&character_physics, &mut animations).iter() {
        let moving = physics.movement_velocity.length() > 0.1;

        let new_state = if physics.is_grounded {
            if moving {
                HumanoidAnimationState::Walk
            } else {
                HumanoidAnimationState::Idle
            }
        } else {
            if physics.vertical_velocity > 0.0 {
                HumanoidAnimationState::Jump
            } else {
                HumanoidAnimationState::Fall
            }
        };

        let current_index = anim.current_state.as_index();
        let current_entry = &anim.mapping.entries[current_index as usize];

        if !current_entry.looping && !anim.finished {
            continue;
        }

        anim.current_state = new_state;
    }
}
