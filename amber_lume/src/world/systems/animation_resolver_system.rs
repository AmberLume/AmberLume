use crate::world::components::animation_blueprint_component::AnimationBlueprintComponent;
use crate::world::components::animation_component::AnimationComponent;
use crate::world::components::animation_parameters_component::AnimationParametersComponent;
use crate::world::components::mesh_component::MeshComponent;
use crate::world::unique::resource_resolver_unique::ResourceResolverUnique;
use animation::blueprint::animation_state_blueprint::AnimationStateBlueprint;
use animation::state_machine::animation_state::AnimationState;
use animation::state_machine::animation_state_machine::AnimationStateMachine;
use anyhow::{bail, Context, Result};
use index_allocator::ResourceId;
use resource_residency::ResourceProvider;
use resource_store::AnimationBackend;
use resource_store::AnimationConfig;
use shipyard::{EntitiesViewMut, Get, IntoIter, Remove, UniqueView, View, ViewMut};
use std::sync::Arc;
use tracing::error;

pub fn animation_resolver_system(
    entities: EntitiesViewMut,
    mesh_components: View<MeshComponent>,
    mut animation_blueprint_components: ViewMut<AnimationBlueprintComponent>,
    mut animation_components: ViewMut<AnimationComponent>,
    mut animation_parameters_components: ViewMut<AnimationParametersComponent>,
    resource_resolver_unique: UniqueView<ResourceResolverUnique>,
) {
    let animation_provider = &resource_resolver_unique.animation_provider;

    let entities_to_resolve = animation_blueprint_components
        .iter()
        .with_id()
        .map(|(entity_id, _)| entity_id)
        .collect::<Vec<_>>();

    for entity_id in entities_to_resolve {
        let Ok(mesh_component) = mesh_components.get(entity_id) else {
            continue;
        };

        let Some(skeleton) = mesh_component.skeleton.as_ref() else {
            animation_blueprint_components.remove(entity_id);

            error!("Animated mesh has no skeleton");

            continue;
        };

        let skeleton_resident = resource_resolver_unique.skeleton_provider
            .with_resource(skeleton.id, |_| ())
            .is_some();

        if !skeleton_resident {
            continue;
        }

        let Some(animation_blueprint) = animation_blueprint_components.remove(entity_id) else {
            continue;
        };

        let states = animation_blueprint
            .states
            .into_iter()
            .map(|state| new_animation_state(animation_provider, state, skeleton.id))
            .collect::<Result<Vec<_>>>();

        let states = match states {
            Ok(states) => states,
            Err(error) => {
                error!("Failed to resolve animations: {:#}", error);

                continue;
            }
        };

        let state_machine = Arc::new(AnimationStateMachine::new(
            states,
            animation_blueprint.transitions,
            animation_blueprint.initial_state,
        ));

        entities.add_component(
            entity_id,
            (
                &mut animation_components,
                &mut animation_parameters_components,
            ),
            (
                AnimationComponent::create(state_machine),
                AnimationParametersComponent::INITIAL,
            ),
        );
    }
}

fn new_animation_state(
    provider: &ResourceProvider<AnimationBackend>,
    blueprint: AnimationStateBlueprint,
    skeleton_id: ResourceId,
) -> Result<AnimationState> {
    let clip = provider.acquire_sync(AnimationConfig::Alpaca {
        resource_key: blueprint.clip.key().to_string(),
    })?;

    let duration = provider
        .with_resource(clip.id, |resource| resource.duration)
        .context("Resolved animation is not available")?;

    let clip_skeleton_id = provider
        .with_resource(clip.id, |resource| resource.skeleton.id)
        .context("Resolved animation is not available")?;

    if clip_skeleton_id != skeleton_id {
        bail!("Animation {} is not made for the mesh skeleton", blueprint.clip.key());
    }

    Ok(AnimationState {
        clip,

        duration,
        speed: blueprint.speed,

        mode: blueprint.mode,
    })
}
