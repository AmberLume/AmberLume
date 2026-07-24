use glam::Vec3;
use physics::SphereSweepHit;
use shipyard::{Get, IntoIter, UniqueView, View, ViewMut};
use crate::world::components::camera_component::CameraComponent;
use crate::world::components::camera_orbit_component::CameraOrbitComponent;
use crate::world::components::position_component::PositionComponent;
use crate::world::components::rotation_component::RotationComponent;
use crate::world::physics::components::physical_body_component::PhysicalBodyComponent;
use crate::world::physics::physics_context_unique::PhysicsContextUnique;

pub fn camera_synchronization_system(
    physics_context_unique: UniqueView<PhysicsContextUnique>,
    physical_bodies: View<PhysicalBodyComponent>,
    cameras: View<CameraComponent>,
    orbits: View<CameraOrbitComponent>,
    rotations: View<RotationComponent>,
    mut positions: ViewMut<PositionComponent>,
) {
    for (camera_id, (camera, orbit)) in (&cameras, &orbits).iter().with_id() {
        let Some(target_id) = camera.target_id else {
            continue;
        };

        let Ok(target_position) = positions.get(target_id).map(|position| position.position) else {
            continue;
        };

        let Ok(rotation) = rotations.get(camera_id).map(|rotation| rotation.rotation) else {
            continue;
        };

        let pivot = target_position + orbit.pivot_offset;
        let forward = (rotation * Vec3::Z).normalize_or_zero();

        let exclude = physical_bodies.get(target_id)
            .ok()
            .map(|physical_body| physical_body.rigid_body_handle);

        let distance = SphereSweepHit::cast(
            &physics_context_unique.handle,
            pivot,
            -forward,
            orbit.collision_radius,
            orbit.distance,
            exclude,
        )
            .map(|hit| hit.distance)
            .unwrap_or(orbit.distance);

        if let Ok(mut position) = (&mut positions).get(camera_id) {
            position.position = pivot - forward * distance;
        }
    }
}
