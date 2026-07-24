use glam::Vec3;
use shipyard::{Get, IntoIter, View, ViewMut};
use crate::world::components::camera_component::CameraComponent;
use crate::world::components::camera_orbit_component::CameraOrbitComponent;
use crate::world::components::position_component::PositionComponent;
use crate::world::components::rotation_component::RotationComponent;

pub fn camera_synchronization_system(
    cameras: View<CameraComponent>,
    orbits: View<CameraOrbitComponent>,
    rotations: View<RotationComponent>,
    mut positions: ViewMut<PositionComponent>,
) {
    for (camera_id, (camera, orbit)) in (&cameras, &orbits).iter().with_id() {
        let Some(target_id) = camera.target_id else {
            continue;
        };

        let Ok(target_position) = positions.get(target_id).map(|position| position.position) else {
            continue;
        };

        let Ok(rotation) = rotations.get(camera_id).map(|rotation| rotation.rotation) else {
            continue;
        };

        let pivot = target_position + orbit.pivot_offset;
        let forward = (rotation * Vec3::Z).normalize_or_zero();

        if let Ok(mut position) = (&mut positions).get(camera_id) {
            position.position = pivot - forward * orbit.distance;
        }
    }
}
