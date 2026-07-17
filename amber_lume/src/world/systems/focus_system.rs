use glam::Vec3;
use shipyard::{Get, IntoIter, UniqueView, View, ViewMut};
use physics::RayHit;
use crate::world::components::camera_component::CameraComponent;
use crate::world::components::focus_component::FocusComponent;
use crate::world::components::position_component::PositionComponent;
use crate::world::components::rotation_component::RotationComponent;
use crate::world::physics::components::physical_body_component::PhysicalBodyComponent;
use crate::world::physics::physics_context_unique::PhysicsContextUnique;

pub fn focus_system(
    physics_context_unique: UniqueView<PhysicsContextUnique>,
    physical_bodies: View<PhysicalBodyComponent>,
    positions: View<PositionComponent>,
    rotations: View<RotationComponent>,
    cameras: View<CameraComponent>,
    mut focuses: ViewMut<FocusComponent>,
) {
    for (position, rotation, camera, focus) in (&positions, &rotations, &cameras, &mut focuses).iter() {
        let origin = position.position;
        let direction = rotation.rotation * Vec3::Z;

        let exclude = camera.target_id
            .and_then(|target_id| physical_bodies.get(target_id).ok())
            .map(|physical_body| physical_body.rigid_body_handle);

        focus.hit = RayHit::cast(&physics_context_unique.handle, origin, direction, focus.max_distance, exclude);
    }
}
