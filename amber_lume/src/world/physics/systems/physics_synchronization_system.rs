use shipyard::{IntoIter, UniqueView, View, ViewMut};
use crate::world::physics::components::physical_body_component::PhysicalBodyComponent;
use crate::world::components::position_component::PositionComponent;
use crate::world::components::rotation_component::RotationComponent;
use crate::world::physics::physics_context_unique::PhysicsContextUnique;

pub fn physics_synchronization_system(
    physical_bodies: View<PhysicalBodyComponent>,
    mut positions: ViewMut<PositionComponent>,
    mut rotations: ViewMut<RotationComponent>,
    physics_context_unique: UniqueView<PhysicsContextUnique>,
) {
    for (position, rotation, physical_body) in (&mut positions, &mut rotations, &physical_bodies).iter() {
        if physical_body.skip_synchronization { continue; }

        let parent_handle = physical_body.rigid_body_handle;

        let (new_position, new_rotation) = parent_handle.interpolated_position_rotation(&physics_context_unique.handle);

        position.position = new_position;
        rotation.rotation = new_rotation;
    }
}
