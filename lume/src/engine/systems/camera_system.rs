use glam::Vec3;
use amber_lume::world::unique::world_camera_unique::WorldCameraUnique;
use shipyard::UniqueViewMut;
use crate::engine::camera_utils::create_view_projection_matrix;

pub fn camera_system(mut world_camera_unique: UniqueViewMut<WorldCameraUnique>) {
    world_camera_unique.projection_matrix = create_view_projection_matrix(
        1.0,
        80.0,
        Vec3::ZERO,
        10.0,
        0.01,
        100.0,
    )
}
