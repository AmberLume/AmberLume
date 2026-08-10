use glam::Vec3;
use physics::{BodyDescriptor, BodyType, ColliderDescriptor};
use rkyv::access;
use rkyv::rancor::Error;
use shipyard::{EntitiesViewMut, IntoIter, Remove, UniqueView, UniqueViewMut, View, ViewMut};
use tracing::warn;
use crate::data::physical_body_data::ArchivedPhysicalBodyData;
use crate::world::physics::components::physical_body_blueprint_component::PhysicalBodyBlueprintComponent;
use crate::world::components::position_component::PositionComponent;
use crate::world::components::rotation_component::RotationComponent;
use crate::world::physics::components::physical_body_component::{PhysicalBodyCollider, PhysicalBodyComponent};
use crate::world::physics::data::{ColliderData, ColliderShape, PhysicalBodyData};
use crate::world::physics::physics_context_unique::PhysicsContextUnique;
use crate::world::unique::resource_loader_unique::ResourceLoaderUnique;

pub fn physics_registration_system(
    entities: EntitiesViewMut,
    positions: View<PositionComponent>,
    rotations: View<RotationComponent>,
    mut blueprints: ViewMut<PhysicalBodyBlueprintComponent>,
    mut physical_bodies: ViewMut<PhysicalBodyComponent>,
    mut physics_context_unique: UniqueViewMut<PhysicsContextUnique>,
    resource_loader_unique: UniqueView<ResourceLoaderUnique>,
) {
    let mut constructed_ids = Vec::new();

    for (entity_id, (position, rotation, blueprint)) in (&positions, &rotations, &blueprints).iter().with_id() {
        let blueprint = &blueprint.physical_body_blueprint;

        let body_descriptor = BodyDescriptor {
            body_type: blueprint.body_type,
            position: position.position,
            rotation: rotation.rotation,
        };

        let rigid_body_handle = physics_context_unique.handle.create_body(&body_descriptor);

        let mut colliders = Vec::new();

        if let Ok(physical_body_data_bytes) = resource_loader_unique.resource_reader.get_resource(&blueprint.physical_body_asset_key) {
            let Ok(physical_body_data) = access::<ArchivedPhysicalBodyData, Error>(&physical_body_data_bytes) else {
                warn!("Failed to parse PhysicalBodyData for {}", &blueprint.physical_body_asset_key);

                continue;
            };

            let physical_body_data = PhysicalBodyData::from_rkyv(physical_body_data);

            for collider in &physical_body_data.colliders {
                let descriptor = collider_descriptor_from(collider, blueprint.scale);

                let handle = physics_context_unique.handle.create_collider(rigid_body_handle, &descriptor);

                if let Some(handle) = handle {
                    colliders.push(
                        PhysicalBodyCollider {
                            handle,

                            position: collider.translation,
                            rotation: collider.rotation,
                        }
                    )
                } else {
                    warn!("Failed to create collider '{}'", collider.name);

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

fn collider_descriptor_from(collider: &ColliderData, scale: Vec3) -> ColliderDescriptor {
    let size = collider.scale * scale;

    let shape = match &collider.shape {
        ColliderShape::Box => physics::ColliderShape::Box { size },
        ColliderShape::Capsule => {
            let radius = size.x.max(size.z) / 2.0;
            let half_height = (size.y / 2.0 - radius).max(0.0);

            physics::ColliderShape::Capsule { half_height, radius }
        }
        ColliderShape::ConvexHull { vertices } => physics::ColliderShape::ConvexHull {
            points: vertices
                .iter()
                .map(|vertex| Vec3::from_array(*vertex) * size)
                .collect(),
        },
    };

    ColliderDescriptor {
        shape,
        offset: collider.translation,
        rotation: collider.rotation,
        density: collider.density,
        friction: collider.friction,
        restitution: collider.restitution,
    }
}
