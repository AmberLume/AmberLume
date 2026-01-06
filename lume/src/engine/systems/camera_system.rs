use amber_lume::world::unique::world_camera_unique::{CameraStamp, WorldCameraUnique};
use shipyard::{IntoIter, UniqueViewMut, View};
use amber_lume::world::components::position_component::PositionComponent;
use amber_lume::world::components::user_controllable_component::UserControllableComponent;

pub fn camera_system(
    positions: View<PositionComponent>,
    user_controllable_component: View<UserControllableComponent>,
    mut world_camera_unique: UniqueViewMut<WorldCameraUnique>,
) {
    for (position, _user_controllable) in (&positions, &user_controllable_component).iter() {
        world_camera_unique.stamp = CameraStamp::new(
            6.0,
            0.6,
            position.position,
            70.0,
            0.1,
            100.0,
        );
    }
}
