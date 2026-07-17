use glam::{Quat, Vec3};
use crate::collider::ColliderShape;

#[derive(Debug, Clone)]
pub struct ColliderDescriptor {
    pub shape: ColliderShape,
    pub offset: Vec3,
    pub rotation: Quat,
    pub density: f32,
    pub friction: f32,
    pub restitution: f32,
}
