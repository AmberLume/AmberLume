use amber_lume::world::components::mesh_blueprint_component::MeshBlueprintComponent;
use amber_lume::world::components::position_component::PositionComponent;
use amber_lume::world::components::rotation_component::RotationComponent;
use glam::{Quat, Vec3};
use shipyard::{AllStoragesViewMut, EntityId, World};
use tracing::info;
use amber_lume::data::scene_data::{BodyTypeData, EntityPlaceholderData};
use amber_lume::physics::body_type::BodyType;
use amber_lume::resources::resource_manifest::scenes;
use amber_lume::resources::scene_loader::SceneLoader;
use amber_lume::world::components::scale_component::ScaleComponent;
use amber_lume::world::components::camera_component::CameraComponent;
use amber_lume::world::components::focus_component::FocusComponent;
use amber_lume::world::components::grab_component::{GrabComponent, GrabParams};
use amber_lume::world::physics::components::character_physics_component::CharacterPhysicsComponent;
use amber_lume::world::physics::components::physical_body_blueprint_component::PhysicalBodyBlueprintComponent;
use amber_lume::world::physics::data::PhysicalBodyBlueprint;

pub fn load_test_scene(world: &World, scene_loader: &SceneLoader) {
    let scene_data = scene_loader.load(scenes::SANDBOX2).expect("Can't find scene 'Scene'");

    info!("Loading scene: {}", scene_data.name);

    for scene_node_data in scene_data.placeholders {
        add_scene_entity(world, scene_node_data);
    }
}

fn add_scene_entity(world: &World, entity_placeholder_data: EntityPlaceholderData) {
    world.run(|mut all_storages: AllStoragesViewMut| {
        let entity_id = all_storages.add_entity(());

        let scale = Vec3::from_array(entity_placeholder_data.scale);

        all_storages.add_component(entity_id, PositionComponent {
            position: Vec3::from_array(entity_placeholder_data.transform),
        });
        all_storages.add_component(entity_id, RotationComponent {
            rotation: Quat::from_array(entity_placeholder_data.rotation),
        });
        all_storages.add_component(entity_id, ScaleComponent {
            scale,
        });

        all_storages.add_component(entity_id, create_physical_body_blueprint_component(
            &entity_placeholder_data.physical_body_type,
            scale,
            entity_placeholder_data.physical_body.value.clone(),
        ));

        let is_character = entity_placeholder_data.name.contains("Character");

        if is_character {
            all_storages.add_component(entity_id, CharacterPhysicsComponent::create(
                0.01,
                0.25,
                45.0,
                10.0,
                30.0,
                10.0,
            ));

            add_camera_entity(&mut all_storages, Some(entity_id));
        } else {
            all_storages.add_component(entity_id, create_mesh_blueprint_component(&entity_placeholder_data));
        }
    });
}

fn create_mesh_blueprint_component(entity_placeholder: &EntityPlaceholderData) -> MeshBlueprintComponent {
    MeshBlueprintComponent::new(entity_placeholder.mesh.value.clone())
}

fn create_physical_body_blueprint_component(
    body_type_data: &BodyTypeData,
    scale: Vec3,
    physical_body_asset_key: String,
) -> PhysicalBodyBlueprintComponent {
    let physical_body_blueprint = PhysicalBodyBlueprint {
        body_type: BodyType::from_data(body_type_data),

        scale,

        physical_body_asset_key,
    };

    PhysicalBodyBlueprintComponent {
        physical_body_blueprint,
    }
}

fn add_camera_entity(all_storages: &mut AllStoragesViewMut, target_id: Option<EntityId>) {
    let entity_id = all_storages.add_entity(());

    all_storages.add_component(entity_id, PositionComponent {
        position: Vec3::ZERO,
    });
    all_storages.add_component(entity_id, RotationComponent {
        rotation: Quat::IDENTITY,
    });
    all_storages.add_component(entity_id, CameraComponent {
        target_id,

        fov: 80.0,
        near: 0.3,
        far: 10000.0,
    });
    all_storages.add_component(entity_id, FocusComponent {
        max_distance: 5.0,
        hit: None,
    });
    all_storages.add_component(entity_id, GrabComponent {
        params: GrabParams {
            distance: 2.5,
            grab_acceleration: 200.0,

            linear_stiffness: 10000.0,
            linear_damping: 500.0,
        },

        grab: None,
    });
}
