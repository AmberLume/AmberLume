use amber_lume::world::components::model_component::ModelComponent;
use amber_lume::world::components::position_component::PositionComponent;
use amber_lume::world::components::rotation_component::RotationComponent;
use glam::{Quat, Vec3};
use shipyard::{AllStoragesViewMut, UniqueViewMut, World};
use tracing::info;
use amber_lume::physics::body_type::BodyType;
use amber_lume::physics::collider_shape::ColliderShape;
use amber_lume::resources::scene_loader::scene_loader::SceneLoader;
use amber_lume::world::components::scale_component::ScaleComponent;
use amber_lume::world::components::user_controllable_component::UserControllableComponent;
use amber_lume::world::physics::components::character_physics_component::CharacterPhysicsComponent;
use amber_lume::world::physics::components::physical_body_blueprint_component::PhysicalBodyBlueprintComponent;
use amber_lume::world::physics::data::{PhysicalBodyBlueprint, PhysicalBodyColliderBlueprint};
use amber_lume::world::unique::world_camera_unique::WorldCameraUnique;
use builder::data::scene_data::{EntityPlaceholderData, PhysicalBodyData};

pub fn load_test_scene(world: &World, scene_loader: &SceneLoader) {
    setup_camera(world);

    let scene_data = scene_loader.load("sandbox_pbr").expect("Can't find scene 'Scene'");

    info!("Loading scene: {}", scene_data.name);

    for scene_node_data in scene_data.placeholders {
        add_scene_entity(world, &scene_node_data);
    }
}

fn setup_camera(world: &World) {
    world.run(|mut camera_unique: UniqueViewMut<WorldCameraUnique>| {
        *camera_unique = WorldCameraUnique::new();
    });
}

fn add_scene_entity(world: &World, entity_placeholder_data: &EntityPlaceholderData) {
    let position = Vec3::from_array(entity_placeholder_data.transform);
    let rotation = Quat::from_array(entity_placeholder_data.rotation);
    let scale = Vec3::from_array(entity_placeholder_data.scale);

    world.run(|mut all_storages: AllStoragesViewMut| {
        let position_component = PositionComponent {
            position,
        };

        let rotation_component = RotationComponent {
            rotation,
        };

        let scale_component = ScaleComponent {
            scale,
        };

        let model_component = create_model_component(&entity_placeholder_data);

        let blueprint_component = create_physical_body_blueprint_component(&entity_placeholder_data.physical_body);

        let entity_id = all_storages.add_entity((position_component, rotation_component, scale_component, model_component, blueprint_component));

        if entity_placeholder_data.name.contains("character") {
            let user_controllable_component = UserControllableComponent { };
            let character_physical_component = CharacterPhysicsComponent::create(
                0.01,
                0.25,
                45.0,
                10.0,
            );

            all_storages.add_component(entity_id, user_controllable_component);
            all_storages.add_component(entity_id, character_physical_component);
        }
    });
}

fn create_model_component(entity_placeholder_data: &EntityPlaceholderData) -> ModelComponent {
    let (file_name, asset) = &entity_placeholder_data.asset_key.split_once('#').unwrap();
    let resource_path = format!("{}/{}.model", file_name, asset);

    ModelComponent::new(resource_path)
}

fn create_physical_body_blueprint_component(physical_body_data: &PhysicalBodyData) -> PhysicalBodyBlueprintComponent {
    let colliders = physical_body_data.colliders.iter().map(|collider| {
        PhysicalBodyColliderBlueprint {
            position: Vec3::from_array(collider.position),
            rotation: Quat::from_array(collider.rotation),

            shape: ColliderShape::from_data(&collider.collider_shape),
        }
    }).collect();

    let physical_body_blueprint = PhysicalBodyBlueprint {
        body_type: BodyType::from_data(&physical_body_data.body_type),

        colliders,
    };

    PhysicalBodyBlueprintComponent {
        physical_body_blueprint,
    }
}
