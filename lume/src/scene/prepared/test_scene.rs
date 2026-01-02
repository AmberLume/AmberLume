use amber_lume::world::components::model_component::ModelComponent;
use amber_lume::world::components::position_component::PositionComponent;
use amber_lume::world::components::rotation_component::RotationComponent;
use glam::{Quat, Vec3};
use shipyard::{AllStoragesViewMut, UniqueViewMut, World};
use tracing::info;
use alpaca::data::common::scene_data::SceneNodeData;
use amber_lume::resources::scene_loader::scene_loader::SceneLoader;
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
    world.run(|mut all_storages: AllStoragesViewMut| {
        let position_component = PositionComponent {
            position: Vec3::from_array(scene_node_data.transform),
        };

        let rotation_component = RotationComponent {
            quaternion: Quat::from_array(scene_node_data.rotation),
        };

        let (file_name, asset) = &scene_node_data.asset_key.split_once('#').unwrap();
        let resource_path = format!("assets/models/{}/{}.manifest", file_name, asset);
        let model_component = ModelComponent::new(resource_path);

        all_storages.add_entity((position_component, rotation_component, model_component));
    });
}
