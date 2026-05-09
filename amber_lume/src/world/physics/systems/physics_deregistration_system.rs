use shipyard::{UniqueViewMut, View};
use crate::world::physics::components::physical_body_component::PhysicalBodyComponent;
use crate::world::physics::physics_world_unique::PhysicsWorldUnique;

pub fn physics_deregistration_system(
    physical_bodies: View<PhysicalBodyComponent>,
    mut physics_world_unique: UniqueViewMut<PhysicsWorldUnique>,
) {
    for (_entity_id, physical_body) in physical_bodies.deleted() {
        physics_world_unique.handle.remove(physical_body.rigid_body_handle);
    }
}
