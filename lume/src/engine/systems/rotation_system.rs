use amber_lume::world::components::rotation_component::RotationComponent;
use amber_lume::world::unique::world_time_unique::WorldTimeUnique;
use glam::Quat;
use shipyard::{IntoIter, UniqueView, ViewMut};

pub fn rotation_system(
    mut rotation: ViewMut<RotationComponent>,
    world_time_unique: UniqueView<WorldTimeUnique>,
) {
    for rotation in (&mut rotation).iter() {
        let rotation_delta = 5.0 * world_time_unique.delta;

        rotation.quaternion *= Quat::from_rotation_y(rotation_delta);
    }
}
