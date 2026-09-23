use render_snapshot::DebugLine;
use render_snapshot::{AnimationPose, EntityAnimation, RenderEntity, RenderEntityId, RenderSnapshot};
use crate::world::components::position_component::PositionComponent;
use crate::world::components::rotation_component::RotationComponent;
use crate::world::unique::render_snapshot_unique::RenderSnapshotUnique;
use glam::Mat4;
use shipyard::{Get, IntoIter, UniqueView, UniqueViewMut, View};
use crate::world::components::animation_component::AnimationComponent;
use crate::world::components::mesh_component::MeshComponent;
use crate::world::components::scale_component::ScaleComponent;
use animation::playback::animation_playback::AnimationPlayback;
use crate::world::physics::physics_context_unique::PhysicsContextUnique;
use crate::world::unique::global_shadow_unique::GlobalShadowUnique;
use crate::world::unique::render_view_unique::RenderViewUnique;
use crate::world::unique::resource_resolver_unique::ResourceResolverUnique;
use crate::world::unique::terrain_unique::TerrainUnique;
use crate::world::components::outline_component::OutlineComponent;
use crate::world::unique::world_time_unique::WorldTimeUnique;

pub fn render_snapshot_system(
    (positions, rotations, scale): (View<PositionComponent>, View<RotationComponent>, View<ScaleComponent>),
    meshes: View<MeshComponent>,
    animations: View<AnimationComponent>,
    render_view_unique: UniqueView<RenderViewUnique>,
    global_shadow_unique: UniqueView<GlobalShadowUnique>,
    world_time_unique: UniqueView<WorldTimeUnique>,
    physics_context_unique: UniqueView<PhysicsContextUnique>,
    (mut terrain_unique, resource_resolver_unique): (UniqueViewMut<TerrainUnique>, UniqueView<ResourceResolverUnique>),
    outlines: View<OutlineComponent>,
    mut snapshot_unique: UniqueViewMut<RenderSnapshotUnique>,
) {
    let mut entities = Vec::new();

    for (entity_id, (position, rotation, scale, mesh)) in (&positions, &rotations, &scale, &meshes).iter().with_id() {
        let transform_matrix = Mat4::from_scale_rotation_translation(
            scale.scale,
            rotation.rotation,
            position.position,
        );

        let animation = animations.get(entity_id).ok().and_then(|animation| {
            let skeleton = mesh.skeleton.as_ref()?;
            let states = &animation.state_machine.states;

            let pose = |playback: &AnimationPlayback| AnimationPose {
                animation_id: states[playback.current_state as usize].clip.id.inner,
                time: playback.time,

                blend_from_animation_id: states[playback.blend_from_state as usize].clip.id.inner,
                blend_from_time: playback.blend_from_time,
                blend_factor: playback.blend_factor(),
            };

            Some(EntityAnimation {
                skeleton_id: skeleton.id.inner,

                pose: pose(&animation.playback),
                previous_pose: pose(&animation.previous_playback),
            })
        });

        let outline = outlines
            .get(entity_id)
            .map(|outline| outline.color)
            .unwrap_or([0.0; 4]);

        let world_entity = RenderEntity {
            id: RenderEntityId(entity_id.inner()),

            transform_matrix,

            mesh_id: mesh.handle.id.inner,
            animation,
            outline,
        };

        entities.push(world_entity);
    }

    terrain_unique.terrain.append_drawables(&mut entities);

    let debug_lines = physics_context_unique.debug_renderer.lines()
        .iter()
        .map(|line| DebugLine {
            start: line.start,
            end: line.end,
            color: line.color,
        })
        .collect();

    let geometry_changes = resource_resolver_unique.mesh_table.take_geometry_changes();

    snapshot_unique.snapshot = Some(RenderSnapshot {
        camera: render_view_unique.resolved_camera,
        global_shadows_direction: global_shadow_unique.direction,
        global_shadows_color: global_shadow_unique.color,
        global_shadows_intensity: global_shadow_unique.intensity,
        global_ibl_intensity: global_shadow_unique.ibl_intensity,

        time: world_time_unique.elapsed,

        entities,

        geometry_changes,

        debug_lines,
    });
}
