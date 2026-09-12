use crate::world::components::animation_blueprint_component::AnimationBlueprintComponent;
use crate::world::components::animation_component::AnimationComponent;
use crate::world::components::animation_parameters_component::AnimationParametersComponent;
use crate::world::components::animation_render_component::AnimationRenderComponent;
use crate::world::components::mesh_component::MeshComponent;
use crate::world::components::skeleton_component::SkeletonComponent;
use crate::world::unique::resource_resolver_unique::ResourceResolverUnique;
use animation::blueprint::animation_state_blueprint::AnimationStateBlueprint;
use animation::state_machine::animation_state::AnimationState;
use animation::state_machine::animation_state_machine::AnimationStateMachine;
use anyhow::{Context, Result};
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
    mut animation_render_components: ViewMut<AnimationRenderComponent>,
    mut skeleton_components: ViewMut<SkeletonComponent>,
    resource_resolver_unique: UniqueView<ResourceResolverUnique>,
) {
    let animation_provider = &resource_resolver_unique.animation_provider;

    let entities_to_resolve = animation_blueprint_components
        .iter()
        .with_id()
        .map(|(entity_id, _)| entity_id)
        .collect::<Vec<_>>();

    for entity_id in entities_to_resolve {
        let mesh_component = mesh_components.get(entity_id).unwrap();
        let skeleton_id = mesh_component.skeleton.as_ref().unwrap().id;
        let skeleton_bone_count = resource_resolver_unique.skeleton_provider
            .with_resource(skeleton_id, |skeleton| skeleton.bones_allocation.size);

        let Some(skeleton_bone_count) = skeleton_bone_count else {
            continue;
        };

        let animation_blueprint = animation_blueprint_components
            .remove(entity_id)
            .unwrap();

        let states = animation_blueprint
            .states
            .into_iter()
            .map(|state| new_animation_state(animation_provider, state))
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

        let animation_id = state_machine.states[state_machine.initial_state as usize]
            .clip
            .id
            .inner;

        entities.add_component(
            entity_id,
            (
                &mut animation_components,
                &mut animation_parameters_components,
                &mut animation_render_components,
                &mut skeleton_components,
            ),
            (
                AnimationComponent::create(state_machine),
                AnimationParametersComponent::INITIAL,
                AnimationRenderComponent {
                    animation_id,
                    time: 0.0,

                    previous_animation_id: animation_id,
                    previous_time: 0.0,
                    blend_factor: 1.0,
                },
                SkeletonComponent {
                    handle: mesh_component.skeleton.as_ref().unwrap().clone(),

                    bone_transform_allocation: resource_resolver_unique
                        .bone_transform_handler
                        .allocate(skeleton_bone_count),
                },
            ),
        );
    }
}

fn new_animation_state(
    provider: &ResourceProvider<AnimationBackend>,
    blueprint: AnimationStateBlueprint,
) -> Result<AnimationState> {
    let clip = provider.acquire_sync(AnimationConfig::Alpaca {
        resource_key: blueprint.clip.key().to_string(),
    })?;

    let duration = provider
        .with_resource(clip.id, |resource| resource.duration)
        .context("Resolved animation is not available")?;

    Ok(AnimationState {
        clip,

        duration,
        speed: blueprint.speed,

        mode: blueprint.mode,
    })
}
