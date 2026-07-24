use glam::Vec3;
use physics::SphereSweepHit;
use shipyard::{Get, IntoIter, UniqueView, View, ViewMut};
use crate::world::components::camera_component::CameraComponent;
use crate::world::components::camera_orbit_component::CameraOrbitComponent;
use crate::world::components::position_component::PositionComponent;
use crate::world::components::rotation_component::RotationComponent;
use crate::world::physics::components::physical_body_component::PhysicalBodyComponent;
use crate::world::physics::physics_context_unique::PhysicsContextUnique;
use crate::world::unique::world_time_unique::WorldTimeUnique;

pub fn camera_synchronization_system(
    physics_context_unique: UniqueView<PhysicsContextUnique>,
    world_time_unique: UniqueView<WorldTimeUnique>,
    physical_bodies: View<PhysicalBodyComponent>,
    cameras: View<CameraComponent>,
    mut orbits: ViewMut<CameraOrbitComponent>,
    rotations: View<RotationComponent>,
    mut positions: ViewMut<PositionComponent>,
) {
    for (camera_id, (camera, orbit)) in (&cameras, &mut orbits).iter().with_id() {
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

        let obstacle_distance = SphereSweepHit::cast(
            &physics_context_unique.handle,
            pivot,
            -forward,
            orbit.collision_radius,
            orbit.distance.max(orbit.current_distance),
            exclude,
        )
            .map(|hit| hit.distance);

        let target_distance = obstacle_distance
            .map(|distance| distance.min(orbit.distance))
            .unwrap_or(orbit.distance);

        let factor = 1.0 - (-orbit.smoothing_speed * world_time_unique.delta).exp();

        orbit.current_distance += (target_distance - orbit.current_distance) * factor;

        if let Some(obstacle_distance) = obstacle_distance {
            orbit.current_distance = orbit.current_distance.min(obstacle_distance);
        }

        if let Ok(mut position) = (&mut positions).get(camera_id) {
            position.position = pivot - forward * orbit.current_distance;
        }
    }
}
