use crate::world::unique::world_time_unique::WorldTimeUnique;
use shipyard::{IntoIter, UniqueViewMut, ViewMut};
use crate::world::components::animation_component::AnimationComponent;

pub fn animation_system(
    world_time_unique: UniqueViewMut<WorldTimeUnique>,
    mut animation_components: ViewMut<AnimationComponent>,
) {
    for animation in (&mut animation_components).iter() {
        animation.time += world_time_unique.delta;
    }
}
