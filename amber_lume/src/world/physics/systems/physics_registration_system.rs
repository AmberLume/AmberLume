use glam::{Quat, Vec3};
use rkyv::access;
use rkyv::rancor::Error;
use shipyard::{EntitiesViewMut, IntoIter, Remove, UniqueView, UniqueViewMut, View, ViewMut};
use tracing::warn;
use builder::data::physical_body_data::ArchivedPhysicalBodyData;
use crate::physics::body_type::BodyType;
use crate::world::physics::components::physical_body_blueprint_component::PhysicalBodyBlueprintComponent;
use crate::world::components::position_component::PositionComponent;
use crate::world::components::rotation_component::RotationComponent;
use crate::world::physics::components::physical_body_component::{PhysicalBodyCollider, PhysicalBodyComponent};
use crate::world::physics::data::PhysicalBodyData;
use crate::world::physics::physics_world_unique::PhysicsWorldUnique;
use crate::world::unique::resource_loader_unique::ResourceLoaderUnique;

pub fn physics_registration_system(
    entities: EntitiesViewMut,
    positions: View<PositionComponent>,
    rotations: View<RotationComponent>,
    mut blueprints: ViewMut<PhysicalBodyBlueprintComponent>,
    mut physical_bodies: ViewMut<PhysicalBodyComponent>,
    mut physics_world_unique: UniqueViewMut<PhysicsWorldUnique>,
    resource_loader_unique: UniqueView<ResourceLoaderUnique>,
) {
    let mut constructed_ids = Vec::new();

    for (entity_id, (position, rotation, blueprint)) in (&positions, &rotations, &blueprints).iter().with_id() {
        let blueprint = &blueprint.physical_body_blueprint;

        let rigid_body_handle = physics_world_unique.handle.create_parent(&blueprint.body_type, &position.position, &rotation.rotation);

        let mut colliders = Vec::new();

        if let Ok(physical_body_data_bytes) = resource_loader_unique.alpaca_resource_reader.get_resource(&blueprint.physical_body_asset_key) {
            let Ok(physical_body_data) = access::<ArchivedPhysicalBodyData, Error>(&physical_body_data_bytes) else {
                warn!("Failed to parse PhysicalBodyData for {}", &blueprint.physical_body_asset_key);

                continue;
            };

            let physical_body_data = PhysicalBodyData::from_rkyv(physical_body_data);

            for collider in &physical_body_data.colliders {
                let handle = physics_world_unique.handle.add_collider(rigid_body_handle, &blueprint, &collider);

                if let Some(handle) = handle {
                    colliders.push(
                        PhysicalBodyCollider {
                            handle,

                            position: Vec3::from_array(collider.translation),
                            rotation: Quat::from_array(collider.rotation),
                        }
                    )
                } else {
                    continue
                }
            }
        } else {
            warn!("Failed to load PhysicalBodyData for {}", &blueprint.physical_body_asset_key);
        }

        let physical_body_component = PhysicalBodyComponent {
            rigid_body_handle,

            colliders,

            skip_synchronization: blueprint.body_type == BodyType::Static,
        };

        entities.add_component(entity_id, &mut physical_bodies, physical_body_component);
        constructed_ids.push(entity_id);
    }

    for constructed_id in constructed_ids {
        blueprints.remove(constructed_id);
    }
}
