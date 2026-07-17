use shipyard::{UniqueViewMut, View};
use crate::world::physics::components::physical_body_component::PhysicalBodyComponent;
use crate::world::physics::physics_context_unique::PhysicsContextUnique;

pub fn physics_deregistration_system(
    physical_bodies: View<PhysicalBodyComponent>,
    mut physics_context_unique: UniqueViewMut<PhysicsContextUnique>,
) {
    for (_entity_id, physical_body) in physical_bodies.deleted() {
        physical_body.rigid_body_handle.remove(&mut physics_context_unique.handle);
    }
}
