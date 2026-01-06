use amber_lume::world::components::model_component::ModelComponent;
use amber_lume::world::components::position_component::PositionComponent;
use amber_lume::world::components::rotation_component::RotationComponent;
use glam::{Quat, Vec3};
use shipyard::{AllStoragesViewMut, UniqueViewMut, World};
use tracing::info;
use alpaca::data::common::scene_data::{PhysicalBodyData, SceneNodeData};
use amber_lume::physics::body_type::BodyType;
use amber_lume::physics::collider_shape::ColliderShape;
use amber_lume::resources::scene_loader::scene_loader::SceneLoader;
use amber_lume::world::components::user_controllable_component::UserControllableComponent;
use amber_lume::world::physics::components::character_physics_component::CharacterPhysicsComponent;
use amber_lume::world::physics::components::physical_body_blueprint_component::PhysicalBodyBlueprintComponent;
use amber_lume::world::physics::data::{PhysicalBodyBlueprint, PhysicalBodyColliderBlueprint};
use amber_lume::world::unique::world_camera_unique::WorldCameraUnique;

pub fn load_test_scene(world: &World, scene_loader: &SceneLoader) {
    setup_camera(world);

    let scene_data = scene_loader.load("test_level/Scene").expect("Can't find scene 'Scene'");

    info!("Loading scene: {}", scene_data.name);

    for scene_node_data in scene_data.nodes {
        add_scene_entity(world, &scene_node_data);
    }
}

fn setup_camera(world: &World) {
    world.run(|mut camera_unique: UniqueViewMut<WorldCameraUnique>| {
        *camera_unique = WorldCameraUnique::new();
    });
}

fn add_scene_entity(world: &World, scene_node_data: &SceneNodeData) {
    let position = Vec3::from_array(scene_node_data.transform);
    let rotation = Quat::from_array(scene_node_data.rotation);

    world.run(|mut all_storages: AllStoragesViewMut| {
        let position_component = PositionComponent {
            position,
        };

        let rotation_component = RotationComponent {
            rotation,
        };

        let model_component = create_model_component(&scene_node_data);

        let blueprint_component = create_physical_body_blueprint_component(&scene_node_data.physical_body);

        let entity_id = all_storages.add_entity((position_component, rotation_component, model_component, blueprint_component));

        if scene_node_data.name.contains("character") {
            let user_controllable_component = UserControllableComponent { };
            let character_physical_component = CharacterPhysicsComponent::create(
                0.05,
                0.25,
                45.0,
                10.0,
            );

            all_storages.add_component(entity_id, user_controllable_component);
            all_storages.add_component(entity_id, character_physical_component);
        }
    });
}

fn create_model_component(scene_node_data: &SceneNodeData) -> ModelComponent {
    let (file_name, asset) = &scene_node_data.asset_key.split_once('#').unwrap();
    let resource_path = format!("assets/models/{}/{}.manifest", file_name, asset);

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
