use amber_lume::world::components::mesh_blueprint_component::MeshBlueprintComponent;
use amber_lume::world::components::position_component::PositionComponent;
use amber_lume::world::components::rotation_component::RotationComponent;
use glam::{Quat, Vec3};
use shipyard::{AllStoragesViewMut, World};
use tracing::info;
use amber_lume::data::scene_data::{BodyTypeData, EntityPlaceholderData};
use amber_lume::physics::body_type::BodyType;
use amber_lume::resources::scene_loader::SceneLoader;
use amber_lume::world::components::animation_component::AnimationBlueprintComponent;
use amber_lume::world::components::scale_component::ScaleComponent;
use amber_lume::world::components::camera_component::CameraComponent;
use amber_lume::world::components::user_controllable_component::UserControllableComponent;
use amber_lume::world::physics::components::character_physics_component::CharacterPhysicsComponent;
use amber_lume::world::physics::components::physical_body_blueprint_component::PhysicalBodyBlueprintComponent;
use amber_lume::world::physics::data::PhysicalBodyBlueprint;

pub fn load_test_scene(world: &World, scene_loader: &SceneLoader) {
    let scene_data = scene_loader.load("Sandbox2").expect("Can't find scene 'Scene'");

    info!("Loading scene: {}", scene_data.name);

    for scene_node_data in scene_data.placeholders {
        add_scene_entity(world, scene_node_data);
    }
}

fn add_scene_entity(world: &World, entity_placeholder_data: EntityPlaceholderData) {
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

        let mesh_component = create_mesh_blueprint_component(&entity_placeholder_data);

        let physical_body_blueprint_component = create_physical_body_blueprint_component(
            entity_placeholder_data.physical_body_type,
            scale,
            entity_placeholder_data.physical_body.value,
        );

        let entity_id = all_storages.add_entity((position_component, rotation_component, scale_component, mesh_component, physical_body_blueprint_component));

        if entity_placeholder_data.name.contains("Character") {
            let user_controllable_component = UserControllableComponent { };
            let character_physical_component = CharacterPhysicsComponent::create(
                0.01,
                0.25,
                45.0,
                10.0,
            );
            let animation_component = AnimationBlueprintComponent::Humanoid;
            let camera_component = CameraComponent {
                offset: Vec3::new(0.0, 1.7, 0.1),

                yaw: 0.0,
                pitch: 0.0,
                sensitivity: 0.001,

                fov: 80.0,
                near: 0.3,
                far: 10000.0,
            };

            all_storages.add_component(entity_id, user_controllable_component);
            all_storages.add_component(entity_id, character_physical_component);
            all_storages.add_component(entity_id, animation_component);
            all_storages.add_component(entity_id, camera_component);
        }
    });
}

fn create_mesh_blueprint_component(entity_placeholder: &EntityPlaceholderData) -> MeshBlueprintComponent {
    MeshBlueprintComponent::new(entity_placeholder.mesh.value.clone())
}

fn create_physical_body_blueprint_component(
    body_type_data: BodyTypeData,
    scale: Vec3,
    physical_body_asset_key: String,
) -> PhysicalBodyBlueprintComponent {
    let physical_body_blueprint = PhysicalBodyBlueprint {
        body_type: BodyType::from_data(&body_type_data),

        scale,

        physical_body_asset_key,
    };

    PhysicalBodyBlueprintComponent {
        physical_body_blueprint,
    }
}
