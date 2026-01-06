use shipyard::{EntitiesViewMut, IntoIter, Remove, UniqueViewMut, View, ViewMut};
use crate::physics::body_type::BodyType;
use crate::world::physics::components::physical_body_blueprint_component::PhysicalBodyBlueprintComponent;
use crate::world::components::position_component::PositionComponent;
use crate::world::components::rotation_component::RotationComponent;
use crate::world::physics::components::physical_body_component::{PhysicalBodyCollider, PhysicalBodyComponent};
use crate::world::physics::physics_world_unique::PhysicsWorldUnique;

pub fn physics_registration_system(
    entities: EntitiesViewMut,
    positions: View<PositionComponent>,
    rotations: View<RotationComponent>,
    mut blueprints: ViewMut<PhysicalBodyBlueprintComponent>,
    mut physical_bodies: ViewMut<PhysicalBodyComponent>,
    mut physics_world_unique: UniqueViewMut<PhysicsWorldUnique>,
) {
    let mut constructed_ids = Vec::new();

    for (entity_id, (position, rotation, blueprint)) in (&positions, &rotations, &blueprints).iter().with_id() {
        let blueprint = &blueprint.physical_body_blueprint;

        let rigid_body_handle = physics_world_unique.handle.create_parent(&blueprint.body_type, &position.position, &rotation.rotation);

        let colliders = blueprint.colliders.iter().map(|collider| {
            let handle = physics_world_unique.handle.add_collider(rigid_body_handle, &collider.position, &collider.rotation, &collider.shape);
            
            PhysicalBodyCollider {
                handle,
                
                position: collider.position,
                rotation: collider.rotation,
                half_extents: collider.shape.half_extents,
                shape: collider.shape,
            }
        }).collect::<Vec<_>>();

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
