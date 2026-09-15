use crate::world::components::animation_component::AnimationComponent;
use crate::world::components::animation_parameters_component::AnimationParametersComponent;
use crate::world::unique::world_time_unique::WorldTimeUnique;
use animation::parameters::animation_parameters::AnimationParameters;
use shipyard::{IntoIter, UniqueView, View, ViewMut};

pub fn animation_system(
    world_time: UniqueView<WorldTimeUnique>,
    animation_parameters: View<AnimationParametersComponent>,
    mut animations: ViewMut<AnimationComponent>,
) {
    for (parameters, animation) in (&animation_parameters, &mut animations).iter() {
        let parameters = AnimationParameters {
            floats: &parameters.floats,
            flags: &parameters.flags,
        };

        animation.previous_playback = animation.playback;
        animation.playback.advance(&animation.state_machine, &parameters, world_time.delta);
    }
}
