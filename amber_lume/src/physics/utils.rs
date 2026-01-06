use glam::{EulerRot, Quat, Vec3};
use nalgebra::Vector3;
use rapier3d::geometry::SharedShape;
use rapier3d::math::AngVector;
use crate::physics::collider_shape::ColliderShape;

pub fn vector3_from_vec3(vec3: &Vec3) -> Vector3<f32> {
    Vector3::new(vec3.x, vec3.y, vec3.z)
}

pub fn euler_from_quat(quat: &Quat) -> AngVector<f32> {
    let euler_rotation = quat.to_euler(EulerRot::XYZ);

    AngVector::new(euler_rotation.0, euler_rotation.1, euler_rotation.2)
}

pub fn shape_from(collider_shape: &ColliderShape) -> SharedShape {
    match collider_shape {
        ColliderShape::Box { size } => SharedShape::cuboid(size[0] / 2.0, size[1] / 2.0, size[2] / 2.0)
    }
}
