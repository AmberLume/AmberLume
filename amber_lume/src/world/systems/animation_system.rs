use crate::world::unique::world_time_unique::WorldTimeUnique;
use shipyard::{IntoIter, UniqueView, ViewMut};
use crate::animation::animation_state::AnimationState;
use crate::animation::play_mode::PlayMode;
use crate::world::components::animation_component::AnimationComponent;
use crate::world::components::animation_render_component::AnimationRenderComponent;

pub fn animation_system<S: AnimationState + Send + Sync>(
    world_time: UniqueView<WorldTimeUnique>,
    mut animation_components: ViewMut<AnimationComponent<S>>,
    mut animation_render_components: ViewMut<AnimationRenderComponent>,
) {
    for (animation, render) in (&mut animation_components, &mut animation_render_components).iter() {
        let current_state = animation.current_state;

        if current_state != animation.last_state {
            animation.blend_from_state = animation.last_state;
            animation.blend_from_time = animation.time;

            animation.last_state = current_state;

            animation.time = 0.0;
            animation.finished = false;

            animation.blending = true;
            animation.blend_elapsed = 0.0;
        }

        let entry = &animation.mapping.entries[current_state.as_index() as usize];

        if !animation.finished {
            animation.time += world_time.delta * entry.speed;

            match entry.mode {
                PlayMode::Loop => {
                    if entry.duration > 0.0 {
                        animation.time %= entry.duration;
                    }
                }
                PlayMode::Once { next } | PlayMode::OnceCancellable { next } => {
                    if animation.time >= entry.duration {
                        animation.blend_from_state = animation.current_state;
                        animation.blend_from_time = entry.duration;

                        animation.current_state = S::from_index(next);
                        animation.last_state = animation.current_state;

                        animation.time = 0.0;
                        animation.finished = false;

                        animation.blending = true;
                        animation.blend_elapsed = 0.0;
                    }
                }
            }
        }

        let entry = &animation.mapping.entries[animation.current_state.as_index() as usize];

        let blend_factor;

        if animation.blending {
            animation.blend_elapsed += world_time.delta;

            if animation.blend_elapsed >= animation.blend_duration {
                animation.blending = false;
                blend_factor = 1.0;
            } else {
                blend_factor = animation.blend_elapsed / animation.blend_duration;
            }

            let previous_entry = &animation.mapping.entries[animation.blend_from_state.as_index() as usize];

            match previous_entry.mode {
                PlayMode::Loop => {
                    animation.blend_from_time += world_time.delta * previous_entry.speed;
                    if previous_entry.duration > 0.0 {
                        animation.blend_from_time %= previous_entry.duration;
                    }
                }
                _ => {
                    animation.blend_from_time = animation.blend_from_time.min(previous_entry.duration);
                }
            }

            render.previous_animation_id = previous_entry.handle.id.inner;
            render.previous_time = animation.blend_from_time;
        } else {
            blend_factor = 1.0;

            render.previous_animation_id = entry.handle.id.inner;
            render.previous_time = animation.time;
        }

        render.animation_id = entry.handle.id.inner;
        render.time = animation.time;
        render.blend_factor = blend_factor;
    }
}
