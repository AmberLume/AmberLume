use shipyard::{IntoIter, View, ViewMut};
use animation::HumanoidAnimationState;
use crate::world::components::animation_component::AnimationComponent;
use crate::world::physics::components::character_physics_component::CharacterPhysicsComponent;

pub fn humanoid_animation_system(
    character_physics: View<CharacterPhysicsComponent>,
    mut animations: ViewMut<AnimationComponent<HumanoidAnimationState>>,
) {
    for (physics, animation) in (&character_physics, &mut animations).iter() {
        let moving = physics.movement_velocity.length() > 0.1;

        let new_state = if physics.is_grounded {
            if moving {
                HumanoidAnimationState::Walk
            } else {
                HumanoidAnimationState::Idle
            }
        } else {
            if physics.velocity.y > 0.0 {
                match animation.current_state {
                    HumanoidAnimationState::Jump | HumanoidAnimationState::Fly => continue,
                    _ => HumanoidAnimationState::Jump,
                }
            } else {
                HumanoidAnimationState::Fall
            }
        };

        if !animation.can_interrupt() {
            continue;
        }

        animation.current_state = new_state;
    }
}
