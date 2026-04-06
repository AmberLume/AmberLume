use crate::world::unique::world_time_unique::WorldTimeUnique;
use shipyard::{IntoIter, UniqueView, ViewMut};
use crate::animation::animation_state::AnimationState;
use crate::world::components::animation_component::AnimationComponent;
use crate::world::components::animation_render_component::AnimationRenderComponent;

pub fn animation_system<S: AnimationState + Send + Sync>(
    world_time: UniqueView<WorldTimeUnique>,
    mut animation_components: ViewMut<AnimationComponent<S>>,
    mut animation_render_components: ViewMut<AnimationRenderComponent>,
) {
    for (anim, render) in (&mut animation_components, &mut animation_render_components).iter() {
        let current_index = anim.current_state.as_index();

        if current_index != anim.previous_state_index {
            anim.previous_state_index = current_index;
            anim.time = 0.0;
            anim.finished = false;
        }

        let entry = &anim.mapping.entries[current_index as usize];

        if !anim.finished {
            anim.time += world_time.delta * entry.speed;

            if entry.looping {
                if entry.duration > 0.0 {
                    anim.time %= entry.duration;
                }
            } else {
                if anim.time >= entry.duration {
                    anim.time = entry.duration;
                    anim.finished = true;
                }
            }
        }

        render.animation_id = entry.handle.id;
        render.time = anim.time;
    }
}
