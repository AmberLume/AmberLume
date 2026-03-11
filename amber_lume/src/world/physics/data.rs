use glam::{Quat, Vec3};
use crate::physics::body_type::BodyType;
use crate::physics::collider_shape::ColliderShape;

#[derive(Debug)]
pub struct PhysicalBodyBlueprint {
    pub body_type: BodyType,

    pub scale: Vec3,
    pub colliders: Vec<PhysicalBodyColliderBlueprint>
}

#[derive(Debug)]
pub struct PhysicalBodyColliderBlueprint {
    pub position: Vec3,
    pub rotation: Quat,

    pub density: f32,
    pub friction: f32,
    pub restitution: f32,
    
    pub shape: ColliderShape,
}
