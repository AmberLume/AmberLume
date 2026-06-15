use std::f32::consts::PI;
use crate::world::unique::world_time_unique::WorldTimeUnique;
use shipyard::{UniqueView, UniqueViewMut};
use glam::Vec3;
use crate::world::unique::global_shadow_unique::GlobalShadowUnique;
use crate::world::unique::settings_unique::SettingsUnique;

pub fn global_light_system(
    world_time_unique: UniqueView<WorldTimeUnique>,
    settings_unique: UniqueView<SettingsUnique>,
    mut global_shadow_unique: UniqueViewMut<GlobalShadowUnique>,
) {
    let settings = settings_unique.settings.load();
    let light = &settings.light;

    let direction = if light.auto_day_night.value {
        let day_duration = 30.0;
        let time = world_time_unique.elapsed;
        let angle = (time / day_duration) * 2.0 * PI;

        Vec3::new(angle.cos(), -angle.sin(), angle.cos() * 0.3)
    } else {
        Vec3::new(
            light.direction_x.value,
            light.direction_y.value,
            light.direction_z.value,
        )
    };

    global_shadow_unique.direction = direction.normalize_or_zero();
    global_shadow_unique.color = Vec3::new(
        light.color_r.value,
        light.color_g.value,
        light.color_b.value,
    );
    global_shadow_unique.intensity = light.intensity.value;
    global_shadow_unique.ambient = light.ambient.value;
}
