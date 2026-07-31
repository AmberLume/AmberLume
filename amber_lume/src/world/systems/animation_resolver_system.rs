use std::sync::Arc;
use crate::animation::animation_states::humanoid_animation_state::HumanoidAnimationState;
use crate::world::components::animation_component::{
    AnimationBlueprintComponent, AnimationComponent,
};
use crate::world::components::skeleton_component::SkeletonComponent;
use crate::world::unique::resource_resolver_unique::ResourceResolverUnique;
use shipyard::{EntitiesViewMut, Get, IntoIter, Remove, UniqueView, View, ViewMut};
use crate::animation::animation_mapping::{AnimationMapping, AnimationMappingEntry};
use crate::animation::animation_state::AnimationState;
use crate::animation::play_mode::PlayMode;
use crate::data::resource_handle::AnimationResource;
use crate::resources::resource_manifest::animations;
use crate::resources::store::providers::animation::animation_backend::AnimationBackend;
use crate::resources::store::providers::animation::animation_config::AnimationConfig;
use crate::resources::store::providers::resource_provider::ResourceProvider;
use crate::world::components::animation_render_component::AnimationRenderComponent;
use crate::world::components::mesh_component::MeshComponent;

pub fn animation_resolver_system(
    entities: EntitiesViewMut,
    mesh_components: View<MeshComponent>,
    mut animation_blueprint_components: ViewMut<AnimationBlueprintComponent>,
    mut animation_components: ViewMut<AnimationComponent<HumanoidAnimationState>>,
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
        let skeleton = resource_resolver_unique.skeleton_provider.get_resource(skeleton_id);

        if skeleton.is_none() {
            continue;
        }

        let animation_blueprint = animation_blueprint_components
            .remove(entity_id)
            .unwrap();

        let mapping = match animation_blueprint {
            AnimationBlueprintComponent::Humanoid => build_humanoid_mapping(animation_provider),
        };

        entities.add_component(
            entity_id,
            (
                &mut animation_components,
                &mut animation_render_components,
                &mut skeleton_components,
            ),
            (
                AnimationComponent::<HumanoidAnimationState> {
                    current_state: HumanoidAnimationState::Idle,

                    mapping: mapping.clone(),
                    time: 0.0,
                    finished: false,

                    blend_from_state: HumanoidAnimationState::Idle,
                    blend_from_time: 0.0,
                    blend_elapsed: 0.0,
                    blend_duration: 0.25,
                    blending: false,

                    last_state: HumanoidAnimationState::Idle,
                },
                AnimationRenderComponent {
                    animation_id: mapping.entries[0].handle.id.inner,
                    time: 0.0,

                    previous_animation_id:mapping.entries[0].handle.id.inner,
                    previous_time: 0.0,
                    blend_factor: 1.0,
                },
                SkeletonComponent {
                    handle: mesh_component.skeleton.as_ref().unwrap().clone(),

                    bone_transform_allocation: resource_resolver_unique.bone_transform_handler
                        .allocate(skeleton.unwrap().bones_allocation.size)
                },
            ),
        );
    }
}

fn build_humanoid_mapping(provider: &ResourceProvider<AnimationBackend>) -> Arc<AnimationMapping> {
    Arc::new(AnimationMapping::new::<HumanoidAnimationState>(vec![
        new_animation_entry(provider, animations::IDLE, 1.0, PlayMode::Loop),
        new_animation_entry(provider, animations::WALK, 1.0, PlayMode::Loop),
        new_animation_entry(provider, animations::HELLO, 1.0, PlayMode::OnceCancellable { next: HumanoidAnimationState::Idle.as_index() }),
        new_animation_entry(provider, animations::JUMP, 1.0, PlayMode::Once { next: HumanoidAnimationState::Fly.as_index() }),
        new_animation_entry(provider, animations::FLY, 1.0, PlayMode::Loop),
        new_animation_entry(provider, animations::FALL, 1.0, PlayMode::Loop)
    ]))
}

fn new_animation_entry(
    provider: &ResourceProvider<AnimationBackend>,
    animation: AnimationResource,
    speed: f32,
    mode: PlayMode,
) -> AnimationMappingEntry {
    let handle = provider.acquire_sync(AnimationConfig::Alpaca {
        resource_key: animation.key().to_string(),
    });
    let resource = provider.get_resource(handle.id).unwrap();

    AnimationMappingEntry {
        handle,
        duration: resource.duration,
        speed,
        mode,
    }
}
