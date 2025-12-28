use crate::snapshot_handler::world_snapshot::{WorldEntity, WorldSnapshot};
use crate::world::components::model_component::ModelComponent;
use crate::world::components::position_component::PositionComponent;
use crate::world::components::rotation_component::RotationComponent;
use crate::world::unique::world_camera_unique::WorldCameraUnique;
use crate::world::unique::world_snapshot_unique::WorldSnapshotUnique;
use glam::{Mat4, Vec3};
use shipyard::{IntoIter, UniqueView, UniqueViewMut, View};

pub fn world_snapshot_system(
    position: View<PositionComponent>,
    rotation: View<RotationComponent>,
    model: View<ModelComponent>,
    camera_unique: UniqueView<WorldCameraUnique>,
    snapshot_unique: UniqueViewMut<WorldSnapshotUnique>,
) {
    let mut entities = Vec::new();

    for (position, rotation, model) in (&position, &rotation, &model).iter() {
        let model_id = model
            .model_ref
            .get_id()
            .expect("All ResRefs must be resolver before snapshotting");

        let transform_matrix = Mat4::from_scale_rotation_translation(
            Vec3::ONE,
            rotation.quaternion,
            position.position,
        );

        let world_entity = WorldEntity {
            transform_matrix,

            model_id,
        };

        entities.push(world_entity);
    }

    let world_snapshot = WorldSnapshot {
        camera_stamp: camera_unique.stamp.clone(),

        entities,
    };

    snapshot_unique.handler.push(world_snapshot);
}
