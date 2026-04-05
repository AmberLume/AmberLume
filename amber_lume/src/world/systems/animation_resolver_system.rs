use crate::world::unique::resource_resolver_unique::ResourceResolverUnique;
use shipyard::{EntitiesViewMut, IntoIter, Remove, UniqueView, ViewMut};
use tracing::error;
use crate::world::components::animation_component::{AnimationBlueprintComponent, AnimationComponent};

pub fn animation_resolver_system(
    entities: EntitiesViewMut,
    mut animation_blueprint_components: ViewMut<AnimationBlueprintComponent>,
    mut animation_components: ViewMut<AnimationComponent>,
    resource_resolver_unique: UniqueView<ResourceResolverUnique>,
) {
    let animation_provider = &resource_resolver_unique.animation_provider;

    let entities_to_resolve = animation_blueprint_components
        .iter()
        .with_id()
        .map(|(entity_id, _)| entity_id)
        .collect::<Vec<_>>();

    for entity_id in entities_to_resolve {
        let Some(animation_blueprint) = animation_blueprint_components.remove(entity_id) else {
            error!("Entity to resolve does not have AnimationBlueprint");

            continue;
        };

        let handle = animation_provider.get_or_load(animation_blueprint.config);

        entities.add_component(entity_id, &mut animation_components, AnimationComponent { handle });
    }
}
