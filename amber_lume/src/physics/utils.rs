use glam::{Quat, Vec3};
use nalgebra::{Quaternion, UnitQuaternion, Vector3};
use rapier3d::geometry::SharedShape;
use rapier3d::math::AngVector;
use rapier3d::prelude::Point;
use crate::physics::collider_shape::ShapeType;
use crate::world::physics::data::{PhysicalBodyBlueprint, PhysicalBodyColliderBlueprint};

pub fn vector3_from_vec3(vec3: &Vec3) -> Vector3<f32> {
    Vector3::new(vec3.x, vec3.y, vec3.z)
}

pub fn euler_from_quat(quat: &Quat) -> AngVector<f32> {
    let quat = UnitQuaternion::new_normalize(
        Quaternion::new(quat.w, quat.x, quat.y, quat.z)
    );

    quat.scaled_axis()
}

pub fn shared_shape_from(
    body: &PhysicalBodyBlueprint,
    collider: &PhysicalBodyColliderBlueprint,
) -> Option<SharedShape> {
    match &collider.shape.shape_type {
        ShapeType::Box { size } => Some(
            SharedShape::cuboid(
                size[0] / 2.0 * body.scale.x,
                size[1] / 2.0 * body.scale.y,
                size[2] / 2.0 * body.scale.z,
            )
        ),
        ShapeType::Sphere { radius } => Some(SharedShape::ball(*radius * body.scale.x)),
        ShapeType::ConvexHull { vertices } => {
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
