use shipyard::{IntoIter, UniqueViewMut, View};
use crate::snapshot_handler::resolved_camera::ResolvedCamera;
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
    for (position, rotation, camera) in (&positions, &rotations, &cameras).iter() {
        render_view_unique.resolved_camera = ResolvedCamera::resolve(camera, position.position, rotation.rotation);
    }
}
