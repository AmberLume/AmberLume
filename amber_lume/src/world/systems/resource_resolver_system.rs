use crate::world::components::mesh_blueprint_component::MeshBlueprintComponent;
use crate::world::unique::resource_resolver_unique::ResourceResolverUnique;
use shipyard::{EntitiesViewMut, IntoIter, Remove, UniqueView, ViewMut};
use tracing::error;
use crate::world::components::mesh_component::MeshComponent;

pub fn resource_resolver_system(
    entities: EntitiesViewMut,
    mut mesh_blueprint_components: ViewMut<MeshBlueprintComponent>,
    mut mesh_components: ViewMut<MeshComponent>,
    resource_resolver_unique: UniqueView<ResourceResolverUnique>,
) {
    let mesh_provider = &resource_resolver_unique.mesh_provider;

    let entities_to_resolve = mesh_blueprint_components.iter().with_id()
        .map(|(entity_id, _)| entity_id)
        .collect::<Vec<_>>();

    for entity_id in entities_to_resolve {
        let Some(mesh_blueprint) = mesh_blueprint_components.remove(entity_id) else {
            error!("Entity to resolve does not have MeshBlueprint");

            continue;
        };

        let handle = match mesh_provider.acquire_sync(mesh_blueprint.config) {
            Ok(handle) => handle,
            Err(error) => {
                error!("Failed to resolve mesh: {:#}", error);

                continue;
            }
        };

        let skeleton = mesh_provider.with_resource(handle.id, |mesh| mesh.skeleton.clone());

        let Some(skeleton) = skeleton else {
            error!("Resolved mesh is not available");

            continue;
        };

        entities.add_component(entity_id, &mut mesh_components, MeshComponent {
            handle,

            skeleton,
        });
    }
}
