use amber_lume::world::components::model_component::ModelComponent;
use amber_lume::world::components::position_component::PositionComponent;
use amber_lume::world::components::rotation_component::RotationComponent;
use glam::{Quat, Vec3};
use shipyard::{AllStoragesViewMut, UniqueViewMut, World};
use amber_lume::world::unique::world_camera_unique::WorldCameraUnique;

pub fn load_test_scene(world: &World) {
    setup_camera(world);
    add_character(world);
}

fn setup_camera(world: &World) {
    world.run(|mut camera_unique: UniqueViewMut<WorldCameraUnique>| {
        *camera_unique = WorldCameraUnique::new();
    });
}

fn add_character(world: &World) {
    world.run(|mut all_storages: AllStoragesViewMut| {
        let position_component = PositionComponent {
            position: Vec3::new(0.0, 0.0, 0.0),
        };

        let rotation_component = RotationComponent {
            quaternion: Quat::IDENTITY,
        };

        let model_component = ModelComponent::new(String::from("assets/models/character/character.manifest"));

        all_storages.add_entity((position_component, rotation_component, model_component));
    });
}
