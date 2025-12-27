use crate::world::unique::world_time_unique::WorldTimeUnique;
use shipyard::UniqueViewMut;
use std::time::Instant;

pub fn world_time_system(mut world_time_unique: UniqueViewMut<WorldTimeUnique>) {
    let now = Instant::now();

    let delta = now
        .duration_since(world_time_unique.last_instant)
        .as_secs_f32();
    let scaled_delta = delta * world_time_unique.scale;

    world_time_unique.delta = scaled_delta;
    world_time_unique.elapsed += scaled_delta;

    world_time_unique.last_instant = now;
}
