use crate::transition::animation_condition::AnimationCondition;
use crate::transition::animation_transition_source::AnimationTransitionSource;

pub struct AnimationTransition {
    pub source: AnimationTransitionSource,
    pub target: u16,

    pub conditions: Vec<AnimationCondition>,

    pub blend_duration: f32,
}
