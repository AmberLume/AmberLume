use crate::snapshot_handler::world_snapshot::{WorldCollider, WorldEntity, WorldSnapshot};
use crate::world::components::model_component::ModelComponent;
use crate::world::components::position_component::PositionComponent;
use crate::world::components::rotation_component::RotationComponent;
use crate::world::unique::world_camera_unique::WorldCameraUnique;
use crate::world::unique::world_snapshot_unique::WorldSnapshotUnique;
use glam::{Mat4, Vec3, Vec4};
use shipyard::{IntoIter, UniqueView, UniqueViewMut, View};
use crate::world::components::scale_component::ScaleComponent;
use crate::world::physics::components::physical_body_component::PhysicalBodyComponent;
use crate::world::unique::global_shadow_unique::GlobalShadowUnique;

pub fn world_snapshot_system(
    positions: View<PositionComponent>,
    rotations: View<RotationComponent>,
    scale: View<ScaleComponent>,
    models: View<ModelComponent>,
    physical_bodies: View<PhysicalBodyComponent>,
    camera_unique: UniqueView<WorldCameraUnique>,
    global_shadow_unique: UniqueView<GlobalShadowUnique>,
    snapshot_unique: UniqueViewMut<WorldSnapshotUnique>,
) {
    let mut entities = Vec::new();

    for (position, rotation, scale, model) in (&positions, &rotations, &scale, &models).iter() {
        let Some(model_res_ref) = &model.model_ref else {
            continue;
        };

        let transform_matrix = Mat4::from_scale_rotation_translation(
            scale.scale,
            rotation.rotation,
            position.position,
        );

        let world_entity = WorldEntity {
            transform_matrix,

            model_id: model_res_ref.id,
        };

        entities.push(world_entity);
    }

    let mut colliders = Vec::new();

    for (position, rotation, physical_body) in (&positions, &rotations, &physical_bodies).iter() {
        physical_body.colliders.iter().for_each(|collider| {
            let transform_matrix = Mat4::from_scale_rotation_translation(
                Vec3::ONE,
                rotation.rotation * collider.rotation,
                position.position + collider.position,
            );

            let world_collider = WorldCollider {
                transform_matrix,

                color: Vec4::new(1.0, 0.0, 1.0, 1.0),
            };

            colliders.push(world_collider);
        });
    }

    let world_snapshot = WorldSnapshot {
        camera_stamp: camera_unique.stamp.clone(),
        global_shadows_direction: global_shadow_unique.direction,

        entities,

        colliders,
    };

    snapshot_unique.handler.push(world_snapshot);
}
