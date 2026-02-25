use std::f32::consts::PI;
use crate::world::unique::world_time_unique::WorldTimeUnique;
use shipyard::UniqueViewMut;
use glam::Vec3;
use crate::world::unique::global_shadow_unique::GlobalShadowUnique;

pub fn world_day_night_system(
    world_time_unique: UniqueViewMut<WorldTimeUnique>,
    mut global_shadow_unique: UniqueViewMut<GlobalShadowUnique>,
) {
    let day_duration = 30.0;
    let time = world_time_unique.elapsed;

    let angle = (time / day_duration) * 2.0 * PI;

    let shadow_direction = Vec3::new(
        angle.cos(),
        -angle.sin(),
        angle.cos() * 0.3,
    ).normalize();

    global_shadow_unique.direction = shadow_direction;
}
