use amber_lume::world::components::mesh_blueprint_component::MeshBlueprintComponent;
use amber_lume::world::components::position_component::PositionComponent;
use amber_lume::world::components::rotation_component::RotationComponent;
use glam::{Quat, Vec3};
use shipyard::{AllStoragesViewMut, EntityId, World};
use tracing::info;
use amber_lume::data::component_data::ComponentData;
use amber_lume::data::scene_data::{BodyTypeData, EntityPlaceholderData};
use amber_lume::physics::BodyType;
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
    let scene_data = scene_loader.load(scenes::SANDBOX).expect("Can't find scene 'Scene'");

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

        if let Some(mesh) = &entity_placeholder_data.mesh {
            all_storages.add_component(entity_id, MeshBlueprintComponent::new(mesh.value.clone()));
        }

        for component in &entity_placeholder_data.components {
            match component {
                ComponentData::Character {
                    offset,
                    auto_step_height,
                    max_slope_angle,
                    speed,
                    push_force,
                    jump_velocity,
                } => {
                    all_storages.add_component(entity_id, CharacterPhysicsComponent::create(
                        *offset,
                        *auto_step_height,
                        *max_slope_angle,
                        *speed,
                        *push_force,
                        *jump_velocity,
                    ));
                }
                ComponentData::Camera { fov, near, far } => {
                    add_camera_entity(&mut all_storages, Some(entity_id), *fov, *near, *far);
                }
            }
        }
    });
}

fn create_physical_body_blueprint_component(
    body_type_data: &BodyTypeData,
    scale: Vec3,
    physical_body_asset_key: String,
) -> PhysicalBodyBlueprintComponent {
    let physical_body_blueprint = PhysicalBodyBlueprint {
        body_type: match body_type_data {
            BodyTypeData::Static => BodyType::Static,
            BodyTypeData::Kinematic => BodyType::Kinematic,
            BodyTypeData::Dynamic => BodyType::Dynamic,
        },

        scale,

        physical_body_asset_key,
    };

    PhysicalBodyBlueprintComponent {
        physical_body_blueprint,
    }
}

fn add_camera_entity(
    all_storages: &mut AllStoragesViewMut,
    target_id: Option<EntityId>,
    fov: f32,
    near: f32,
    far: f32,
) {
    let entity_id = all_storages.add_entity(());

    all_storages.add_component(entity_id, PositionComponent {
        position: Vec3::ZERO,
    });
    all_storages.add_component(entity_id, RotationComponent {
        rotation: Quat::IDENTITY,
    });
    all_storages.add_component(entity_id, CameraComponent {
        target_id,

        fov,
        near,
        far,
    });
    all_storages.add_component(entity_id, FocusComponent {
        max_distance: 5.0,
        hit: None,
    });
    all_storages.add_component(entity_id, GrabComponent {
        params: GrabParams {
            distance: 2.5,

            linear_acceleration: 200.0,
            linear_stiffness: 10000.0,
            linear_damping: 500.0,

            angular_acceleration: 500.0,
            angular_stiffness: 10000.0,
            angular_damping: 1000.0,
        },

        grab: None,
    });
}
