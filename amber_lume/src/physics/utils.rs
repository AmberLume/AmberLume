use glam::Vec3;
use rapier3d::geometry::SharedShape;
use rapier3d::math::Vector;
use crate::world::physics::data::{ColliderData, ColliderShape};

pub fn shared_shape_from(
    scale: Vec3,
    collider_data: &ColliderData,
) -> Option<SharedShape> {
    match &collider_data.shape {
        ColliderShape::Box { size } => Some(
            SharedShape::cuboid(
                size[0] / 2.0 * scale.x,
                size[1] / 2.0 * scale.y,
                size[2] / 2.0 * scale.z,
            )
        ),
        ColliderShape::ConvexHull { vertices } => {
            let points = vertices.iter().map(|vertex| {
                Vector::new(
                    vertex[0] * scale.x,
                    vertex[1] * scale.y,
                    vertex[2] * scale.z,
                )
            }).collect::<Vec<_>>();

            SharedShape::convex_hull(points.as_slice())
        }
    }
}
