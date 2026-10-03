use crate::world::components::animation_blueprint_component::AnimationBlueprintComponent;
use crate::world::components::animation_clips_component::AnimationClipsComponent;
use crate::world::components::animation_component::AnimationComponent;
use crate::world::components::animation_parameters_component::AnimationParametersComponent;
use crate::world::components::mesh_component::MeshComponent;
use crate::world::components::skin_component::SkinComponent;
use crate::world::unique::resource_resolver_unique::ResourceResolverUnique;
use animation::blueprint::animation_state_blueprint::AnimationStateBlueprint;
use animation::state_machine::animation_state::AnimationState;
use animation::state_machine::animation_state_machine::AnimationStateMachine;
use anyhow::{bail, Context, Result};
use index_allocator::ResourceId;
use resource_residency::ResRef;
use resource_store::AnimationBackend;
use shipyard::{EntitiesViewMut, Get, IntoIter, Remove, UniqueView, View, ViewMut};
use std::sync::Arc;
use tracing::error;

pub fn animation_resolver_system(
    entities: EntitiesViewMut,
    mesh_components: View<MeshComponent>,
    mut animation_blueprint_components: ViewMut<AnimationBlueprintComponent>,
    mut animation_clips_components: ViewMut<AnimationClipsComponent>,
    mut animation_components: ViewMut<AnimationComponent>,
    mut animation_parameters_components: ViewMut<AnimationParametersComponent>,
    mut skin_components: ViewMut<SkinComponent>,
    resource_resolver_unique: UniqueView<ResourceResolverUnique>,
) {
    let animation_backend = &resource_resolver_unique.animation_provider.backend;

    let entities_to_load = (&animation_blueprint_components, !&animation_clips_components)
        .iter()
        .with_id()
        .map(|(entity_id, _)| entity_id)
        .collect::<Vec<_>>();

    for entity_id in entities_to_load {
        let Ok(animation_blueprint) = (&animation_blueprint_components).get(entity_id) else {
            continue;
        };

        let clips = animation_blueprint
            .states
            .iter()
            .map(|state| resource_resolver_unique.animation_loader.load(state.clip.key()))
            .collect::<Result<Vec<_>>>();

        let clips = match clips {
            Ok(clips) => clips,
            Err(error) => {
                animation_blueprint_components.remove(entity_id);

                error!("Failed to load animations: {:#}", error);

                continue;
            }
        };

        entities.add_component(entity_id, &mut animation_clips_components, AnimationClipsComponent {
            clips,
        });
    }

    let entities_to_resolve = (&animation_blueprint_components, &animation_clips_components)
        .iter()
        .with_id()
        .map(|(entity_id, _)| entity_id)
        .collect::<Vec<_>>();

    for entity_id in entities_to_resolve {
        let Ok(mesh_component) = mesh_components.get(entity_id) else {
            continue;
        };

        let Some(skeleton) = resource_resolver_unique.mesh_provider.backend
            .with_mesh(mesh_component.handle.id, |mesh| mesh.skeletal.as_ref().map(|skeletal| skeletal.skeleton.clone()))
        else {
            continue;
        };

        let Some(skeleton) = skeleton else {
            animation_blueprint_components.remove(entity_id);
            animation_clips_components.remove(entity_id);

            error!("Animated mesh has no skeleton");

            continue;
        };

        let skeleton_resident = resource_resolver_unique.skeleton_provider.backend
            .with_skeleton(skeleton.id, |_| ())
            .is_some();

        if !skeleton_resident {
            continue;
        }

        let Ok(animation_clips) = (&animation_clips_components).get(entity_id) else {
            continue;
        };

        let clips_resident = animation_clips
            .clips
            .iter()
            .all(|clip| animation_backend.with_animation(clip.id, |_| ()).is_some());

        if !clips_resident {
            continue;
        }

        let (Some(animation_blueprint), Some(animation_clips)) = (
            animation_blueprint_components.remove(entity_id),
            animation_clips_components.remove(entity_id),
        ) else {
            continue;
        };

        let states = animation_blueprint
            .states
            .into_iter()
            .zip(animation_clips.clips)
            .map(|(state, clip)| new_animation_state(animation_backend, state, clip, skeleton.id))
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

        let skin = resource_resolver_unique.skin_loader.load(&mesh_component.handle);

        let skin = match skin {
            Ok(skin) => skin,
            Err(error) => {
                error!("Failed to acquire skin: {:#}", error);

                continue;
            }
        };

        entities.add_component(
            entity_id,
            (
                &mut animation_components,
                &mut animation_parameters_components,
                &mut skin_components,
            ),
            (
                AnimationComponent::create(state_machine, skeleton),
                AnimationParametersComponent::INITIAL,
                SkinComponent { handle: skin },
            ),
        );
    }
}

fn new_animation_state(
    animation_backend: &AnimationBackend,
    blueprint: AnimationStateBlueprint,
    clip: Arc<ResRef>,
    skeleton_id: ResourceId,
) -> Result<AnimationState> {
    let (duration, clip_skeleton_id) = animation_backend
        .with_animation(clip.id, |animation| (animation.duration, animation.skeleton.id))
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
