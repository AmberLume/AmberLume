use animation::blueprint::animation_state_blueprint::AnimationStateBlueprint;
use animation::transition::animation_transition::AnimationTransition;
use shipyard::Component;

#[derive(Component)]
pub struct AnimationBlueprintComponent {
    pub states: Vec<AnimationStateBlueprint>,
    pub transitions: Vec<AnimationTransition>,

    pub initial_state: u16,
}
