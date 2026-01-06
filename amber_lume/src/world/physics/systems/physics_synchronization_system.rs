use shipyard::{IntoIter, UniqueView, View, ViewMut};
use crate::world::physics::components::physical_body_component::PhysicalBodyComponent;
use crate::world::components::position_component::PositionComponent;
use crate::world::components::rotation_component::RotationComponent;
use crate::world::physics::physics_world_unique::PhysicsWorldUnique;

pub fn physics_synchronization_system(
    physical_bodies: View<PhysicalBodyComponent>,
    mut positions: ViewMut<PositionComponent>,
    mut rotations: ViewMut<RotationComponent>,
    physics_world_unique: UniqueView<PhysicsWorldUnique>,
) {
    for (position, rotation, physical_body) in (&mut positions, &mut rotations, &physical_bodies).iter() {
        if physical_body.skip_synchronization { continue; }

        let parent_handle = physical_body.rigid_body_handle;

        let (new_position, new_rotation) = physics_world_unique.handle.get_interpolated_position_rotation(parent_handle);

        position.position = new_position;
        rotation.rotation = new_rotation;
    }
}
