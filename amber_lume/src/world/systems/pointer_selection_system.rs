use crate::world::components::outline_component::OutlineComponent;
use crate::world::physics::components::physical_body_component::PhysicalBodyComponent;
use crate::world::physics::physics_context_unique::PhysicsContextUnique;
use crate::world::unique::render_view_unique::RenderViewUnique;
use crate::world::unique::user_input_unique::UserInputUnique;
use glam::{Vec2, Vec4};
use gpu::ViewProjectionMatrix;
use physics::RayHit;
use shipyard::{EntitiesViewMut, IntoIter, Remove, UniqueView, UniqueViewMut, View, ViewMut};

pub fn pointer_selection_system(
    user_input_unique: UniqueViewMut<UserInputUnique>,
    render_view_unique: UniqueView<RenderViewUnique>,
    physics_context_unique: UniqueView<PhysicsContextUnique>,
    physical_bodies: View<PhysicalBodyComponent>,
    entities: EntitiesViewMut,
    mut outlines: ViewMut<OutlineComponent>,
) -> Option<()> {
    let pointer_position = user_input_unique.pointer_position?;
    let viewport = &render_view_unique.viewport;
    let camera = &render_view_unique.resolved_camera;

    if viewport.x <= 0.0 || viewport.y <= 0.0 {
        return None;
    }

    let view_projection = ViewProjectionMatrix::from_view_projection(
        &camera.view(),
        &camera.projection(viewport.x / viewport.y),
    )
    .vulkan_corrected();

    let ndc = pointer_position / viewport * 2.0 - Vec2::ONE;

    let far_point = view_projection.value.inverse() * Vec4::new(ndc.x, ndc.y, 0.0, 1.0);
    let direction = (far_point.truncate() / far_point.w - camera.position).normalize();

    let hovered = RayHit::cast(
        &physics_context_unique.handle,
        camera.position,
        direction,
        camera.far,
        None,
    )
    .and_then(|hit| {
        physical_bodies
            .iter()
            .with_id()
            .find(|(_, physical_body)| physical_body.rigid_body_handle == hit.body)
            .map(|(entity_id, _)| entity_id)
    });

    let outlined = (&outlines)
        .iter()
        .with_id()
        .map(|(entity_id, _)| entity_id)
        .filter(|entity_id| Some(*entity_id) != hovered)
        .collect::<Vec<_>>();

    for entity_id in outlined {
        outlines.remove(entity_id);
    }

    entities.add_component(
        hovered?,
        &mut outlines,
        OutlineComponent {
            color: [1.0, 1.0, 1.0, 0.5],
        },
    );

    Some(())
}
