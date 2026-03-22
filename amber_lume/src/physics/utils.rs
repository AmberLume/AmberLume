use glam::Quat;
use nalgebra::{Quaternion, UnitQuaternion, Vector3};
use rapier3d::geometry::SharedShape;
use rapier3d::math::AngVector;
use rapier3d::prelude::Point;
use crate::world::physics::data::{ColliderData, ColliderShape, PhysicalBodyBlueprint};

pub fn vector3_from_slice(slice: &[f32; 3]) -> Vector3<f32> {
    Vector3::new(slice[0], slice[1], slice[2])
}

pub fn euler_from_slice(slice: &[f32; 4]) -> AngVector<f32> {
    let quat = Quat::from_slice(slice);

    let quat = UnitQuaternion::new_normalize(
        Quaternion::new(quat.w, quat.x, quat.y, quat.z)
    );

    quat.scaled_axis()
}

pub fn shared_shape_from(
    body: &PhysicalBodyBlueprint,
    collider_data: &ColliderData,
) -> Option<SharedShape> {
    match &collider_data.shape {
        ColliderShape::Box { size } => Some(
            SharedShape::cuboid(
                size[0] / 2.0 * body.scale.x,
                size[1] / 2.0 * body.scale.y,
                size[2] / 2.0 * body.scale.z,
            )
        ),
        ColliderShape::ConvexHull { vertices } => {
            let points = vertices.iter().map(|vertex| {
                Point::new(
                    vertex[0] * body.scale.x,
                    vertex[1] * body.scale.y,
                    vertex[2] * body.scale.z,
                )
            }).collect::<Vec<_>>();

            SharedShape::convex_hull(points.as_slice())
        }
    }
}
