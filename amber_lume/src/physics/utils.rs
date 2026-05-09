use glam::{Quat, Vec3};
use rapier3d::geometry::SharedShape;
use rapier3d::math::{AngVector, Vector};
use crate::world::physics::data::{ColliderData, ColliderShape, PhysicalBodyBlueprint};

pub fn vector3_from_slice(slice: &[f32; 3]) -> Vec3 {
    Vec3::from_array(*slice)
}

pub fn euler_from_slice(slice: &[f32; 4]) -> AngVector {
    let quat = Quat::from_slice(slice).normalize();
    let (axis, angle) = quat.to_axis_angle();

    axis * angle
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
                Vector::new(
                    vertex[0] * body.scale.x,
                    vertex[1] * body.scale.y,
                    vertex[2] * body.scale.z,
                )
            }).collect::<Vec<_>>();

            SharedShape::convex_hull(points.as_slice())
        }
    }
}
