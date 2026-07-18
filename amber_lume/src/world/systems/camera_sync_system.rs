use glam::Vec3;
use shipyard::{Get, IntoIter, View, ViewMut};
use crate::world::components::camera_component::CameraComponent;
use crate::world::components::position_component::PositionComponent;

const CAMERA_OFFSET: Vec3 = Vec3::new(0.0, 1.7, 0.1);

pub fn camera_synchronization_system(
    cameras: View<CameraComponent>,
    mut positions: ViewMut<PositionComponent>,
) {
    for (camera_id, camera) in (&cameras).iter().with_id() {
        let Some(target_id) = camera.target_id else {
            continue;
        };

        let Ok(target_position) = positions.get(target_id).map(|position| position.position) else {
            continue;
        };

        if let Ok(mut position) = (&mut positions).get(camera_id) {
            position.position = target_position + CAMERA_OFFSET;
        }
    }
}
