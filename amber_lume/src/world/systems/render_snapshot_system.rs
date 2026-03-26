use crate::snapshot_handler::render_snapshot::{RenderEntity, RenderSnapshot};
use crate::world::components::mesh_component::MeshComponent;
use crate::world::components::position_component::PositionComponent;
use crate::world::components::rotation_component::RotationComponent;
use crate::world::unique::render_snapshot_unique::RenderSnapshotUnique;
use glam::Mat4;
use shipyard::{IntoIter, UniqueView, UniqueViewMut, View};
use crate::world::components::scale_component::ScaleComponent;
use crate::world::physics::physics_world_unique::PhysicsWorldUnique;
use crate::world::unique::global_shadow_unique::GlobalShadowUnique;
use crate::world::unique::render_view_unique::RenderViewUnique;

pub fn render_snapshot_system(
    positions: View<PositionComponent>,
    rotations: View<RotationComponent>,
    scale: View<ScaleComponent>,
    meshes: View<MeshComponent>,
    render_view_unique: UniqueView<RenderViewUnique>,
    global_shadow_unique: UniqueView<GlobalShadowUnique>,
    mut physics_world_unique: UniqueViewMut<PhysicsWorldUnique>,
    snapshot_unique: UniqueViewMut<RenderSnapshotUnique>,
) {
    let mut entities = Vec::new();

    for (position, rotation, scale, mesh) in (&positions, &rotations, &scale, &meshes).iter() {
        let Some(mesh_res_ref) = &mesh.mesh_ref else {
            continue;
        };

        let transform_matrix = Mat4::from_scale_rotation_translation(
            scale.scale,
            rotation.rotation,
            position.position,
        );

        let world_entity = RenderEntity {
            transform_matrix,

            mesh_id: mesh_res_ref.id,
        };

        entities.push(world_entity);
    }

    let physics_debug_lines = physics_world_unique.handle.get_debug_lines();

    snapshot_unique.handler.push(RenderSnapshot {
        camera: render_view_unique.camera_view,
        global_shadows_direction: global_shadow_unique.direction,

        entities,

        physics_debug_lines,
    });
}
