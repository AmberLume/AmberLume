use crate::define_animation_states;

define_animation_states!(HumanoidAnimationState {
    Idle => "Idle",
    Walk => "Walk",
    Hello => "Hello",
    
    Jump => "Jump",
    Fly => "Fly",
    Fall => "Fall",
});
