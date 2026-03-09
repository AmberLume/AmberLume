use glam::{Quat, Vec3};
use nalgebra::{Quaternion, UnitQuaternion, Vector3};
use rapier3d::geometry::SharedShape;
use rapier3d::math::AngVector;
use rapier3d::prelude::Point;
use crate::physics::collider_shape::{ColliderShape, ShapeType};

pub fn vector3_from_vec3(vec3: &Vec3) -> Vector3<f32> {
    Vector3::new(vec3.x, vec3.y, vec3.z)
}

pub fn euler_from_quat(quat: &Quat) -> AngVector<f32> {
    let quat = UnitQuaternion::new_normalize(
        Quaternion::new(quat.w, quat.x, quat.y, quat.z)
    );

    quat.scaled_axis()
}

pub fn shared_shape_from(collider_shape: &ColliderShape, scale: &Vec3) -> Option<SharedShape> {
    match &collider_shape.shape_type {
        ShapeType::Box { size } => Some(
            SharedShape::cuboid(
                size[0] / 2.0 * scale.x,
                size[1] / 2.0 * scale.y,
                size[2] / 2.0 * scale.z,
            )
        ),
        ShapeType::Sphere { radius } => Some(SharedShape::ball(*radius * scale.x)),
        ShapeType::ConvexHull { vertices } => {
            let points = vertices.iter().map(|vertex| {
                Point::new(
                    vertex[0] * scale.x,
                    vertex[1] * scale.y,
                    vertex[2] * scale.z,
                )
            }).collect::<Vec<_>>();

            SharedShape::convex_hull(points.as_slice())
        }
    }
}
