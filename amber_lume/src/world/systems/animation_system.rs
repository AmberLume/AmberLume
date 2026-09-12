use crate::world::components::animation_component::AnimationComponent;
use crate::world::components::animation_parameters_component::AnimationParametersComponent;
use crate::world::components::animation_render_component::AnimationRenderComponent;
use crate::world::unique::world_time_unique::WorldTimeUnique;
use animation::parameters::animation_parameters::AnimationParameters;
use shipyard::{IntoIter, UniqueView, View, ViewMut};

pub fn animation_system(
    world_time: UniqueView<WorldTimeUnique>,
    animation_parameters: View<AnimationParametersComponent>,
    mut animations: ViewMut<AnimationComponent>,
    mut animation_renders: ViewMut<AnimationRenderComponent>,
) {
    for (parameters, animation, render) in (&animation_parameters, &mut animations, &mut animation_renders).iter() {
        let parameters = AnimationParameters {
            floats: &parameters.floats,
            flags: &parameters.flags,
        };

        animation.playback.advance(&animation.state_machine, &parameters, world_time.delta);

        let playback = &animation.playback;
        let states = &animation.state_machine.states;

        render.animation_id = states[playback.current_state as usize].clip.id.inner;
        render.time = playback.time;

        render.previous_animation_id = states[playback.blend_from_state as usize].clip.id.inner;
        render.previous_time = playback.blend_from_time;
        render.blend_factor = playback.blend_factor();
    }
}
