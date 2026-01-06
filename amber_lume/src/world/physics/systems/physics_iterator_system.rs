use shipyard::{UniqueView, UniqueViewMut};
use crate::world::physics::physics_world_unique::PhysicsWorldUnique;
use crate::world::unique::world_time_unique::WorldTimeUnique;

pub fn physics_iterator_system(
    world_time_unique: UniqueView<WorldTimeUnique>,
    mut physics_world_unique: UniqueViewMut<PhysicsWorldUnique>,
) {
    let delta_time = world_time_unique.delta;

    physics_world_unique.iterate_count = physics_world_unique.handle.step(delta_time)
}
