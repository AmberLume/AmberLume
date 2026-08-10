use render_snapshot::DebugLine;
use render_snapshot::{EntityAnimation, RenderEntity, RenderEntityId, RenderSnapshot};
use crate::world::components::position_component::PositionComponent;
use crate::world::components::rotation_component::RotationComponent;
use crate::world::unique::render_snapshot_unique::RenderSnapshotUnique;
use glam::Mat4;
use shipyard::{Get, IntoIter, UniqueView, UniqueViewMut, View};
use crate::world::components::animation_render_component::AnimationRenderComponent;
use crate::world::components::mesh_component::MeshComponent;
use crate::world::components::scale_component::ScaleComponent;
use crate::world::components::skeleton_component::SkeletonComponent;
use crate::world::physics::physics_context_unique::PhysicsContextUnique;
use crate::world::unique::global_shadow_unique::GlobalShadowUnique;
use crate::world::unique::render_view_unique::RenderViewUnique;
use crate::world::unique::world_time_unique::WorldTimeUnique;

pub fn render_snapshot_system(
    (positions, rotations, scale): (View<PositionComponent>, View<RotationComponent>, View<ScaleComponent>),
    meshes: View<MeshComponent>,
    skeletons: View<SkeletonComponent>,
    animation_renders: View<AnimationRenderComponent>,
    render_view_unique: UniqueView<RenderViewUnique>,
    global_shadow_unique: UniqueView<GlobalShadowUnique>,
    world_time_unique: UniqueView<WorldTimeUnique>,
    physics_context_unique: UniqueView<PhysicsContextUnique>,
    mut snapshot_unique: UniqueViewMut<RenderSnapshotUnique>,
) {
    let mut entities = Vec::new();

    for (entity_id, (position, rotation, scale, mesh)) in (&positions, &rotations, &scale, &meshes).iter().with_id() {
        let transform_matrix = Mat4::from_scale_rotation_translation(
            scale.scale,
            rotation.rotation,
            position.position,
        );

        let animation = animation_renders.get(entity_id).map(|animation| {
            let skeleton = skeletons.get(entity_id).unwrap();

            EntityAnimation {
                animation_id: animation.animation_id,
                skeleton_id: mesh.skeleton.as_ref().unwrap().id.inner,
                bone_transform_offset: skeleton.bone_transform_allocation.offset,
                time: animation.time,

                previous_animation_id: animation.previous_animation_id,
                previous_time: animation.previous_time,
                blend_factor: animation.blend_factor,
            }
        }).ok();

        let world_entity = RenderEntity {
            id: RenderEntityId(entity_id.inner()),

            transform_matrix,

            mesh_id: mesh.handle.id.inner,

            animation,
        };

        entities.push(world_entity);
    }

    let debug_lines = physics_context_unique.debug_renderer.lines()
        .iter()
        .map(|line| DebugLine {
            start: line.start,
            end: line.end,
            color: line.color,
        })
        .collect();

    snapshot_unique.snapshot = Some(RenderSnapshot {
        camera: render_view_unique.resolved_camera,
        global_shadows_direction: global_shadow_unique.direction,
        global_shadows_color: global_shadow_unique.color,
        global_shadows_intensity: global_shadow_unique.intensity,
        global_ibl_intensity: global_shadow_unique.ibl_intensity,

        time: world_time_unique.elapsed,

        entities,

        debug_lines,
    });
}
