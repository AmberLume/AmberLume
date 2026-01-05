use glam::Vec3;
use amber_lume::world::unique::world_camera_unique::{CameraStamp, WorldCameraUnique};
use shipyard::UniqueViewMut;

pub fn camera_system(
    mut world_camera_unique: UniqueViewMut<WorldCameraUnique>,
) {
    world_camera_unique.stamp = CameraStamp::new(
        10.0,
        0.7,
        Vec3::new(0.0, 1.0, 0.0),
        80.0,
        0.1,
        1000.0,
    );
}
