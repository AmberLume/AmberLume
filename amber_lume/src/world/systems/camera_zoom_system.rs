use shipyard::{IntoIter, UniqueViewMut, ViewMut};
use crate::world::components::camera_orbit_component::CameraOrbitComponent;
use crate::world::unique::user_input_unique::UserInputUnique;

pub fn camera_zoom_system(
    mut user_input_unique: UniqueViewMut<UserInputUnique>,
    mut orbits: ViewMut<CameraOrbitComponent>,
) {
    let Some(input) = user_input_unique.input.as_mut() else {
        return;
    };

    let Some(scroll) = input.scroll(true) else {
        return;
    };

    if scroll.y == 0.0 {
        return;
    }

    for orbit in (&mut orbits).iter() {
        orbit.distance = (orbit.distance + scroll.y * orbit.zoom_speed)
            .clamp(orbit.min_distance, orbit.max_distance);
    }
}
