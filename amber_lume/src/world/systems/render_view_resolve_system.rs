use shipyard::{IntoIter, UniqueViewMut, View};
use render_snapshot::CameraView;
use crate::world::components::camera_component::CameraComponent;
use crate::world::components::position_component::PositionComponent;
use crate::world::components::rotation_component::RotationComponent;
use crate::world::unique::render_view_unique::RenderViewUnique;

pub fn render_view_resolve_system(
    positions: View<PositionComponent>,
    rotations: View<RotationComponent>,
    cameras: View<CameraComponent>,
    mut render_view_unique: UniqueViewMut<RenderViewUnique>,
) {
    debug_assert!(
        (&positions, &rotations, &cameras).iter().count() <= 1,
        "Expected at most one camera entity, found multiple",
    );

    if let Some((position, rotation, camera)) = (&positions, &rotations, &cameras).iter().next() {
        render_view_unique.resolved_camera = CameraView {
            position: position.position,
            rotation: rotation.rotation,

            fov: camera.fov,
            near: camera.near,
            far: camera.far,
        };
    }
}
