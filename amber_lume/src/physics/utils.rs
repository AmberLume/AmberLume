use rapier3d::geometry::SharedShape;
use rapier3d::math::Vector;
use crate::world::physics::data::{ColliderData, ColliderShape, PhysicalBodyBlueprint};

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
