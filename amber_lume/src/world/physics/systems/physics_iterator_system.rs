use shipyard::{UniqueView, UniqueViewMut};
use crate::world::physics::physics_world_unique::PhysicsWorldUnique;
use crate::world::unique::render_view_unique::RenderViewUnique;
use crate::world::unique::world_time_unique::WorldTimeUnique;

pub fn physics_iterator_system(
    world_time_unique: UniqueView<WorldTimeUnique>,
    mut physics_world_unique: UniqueViewMut<PhysicsWorldUnique>,
    render_view_unique: UniqueView<RenderViewUnique>,
) {
    let delta_time = world_time_unique.delta;

    let step_count = physics_world_unique.handle.step(delta_time);

    if step_count > 0 {
        physics_world_unique.handle.update_debug_lines(&render_view_unique.camera_view.target, 6.0);
    }

    physics_world_unique.iterate_count = step_count;
}
