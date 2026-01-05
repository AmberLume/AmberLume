use glam::{Quat, Vec3};
use rapier3d::prelude::RigidBodyHandle;
use shipyard::Component;

#[derive(Component, Debug)]
pub struct PhysicalBodyComponent {
    pub colliders: Vec<BodyCollider>,
}

#[derive(Debug)]
pub struct BodyCollider {
    pub position: Vec3,
    pub rotation: Quat,
    
    pub handle: Option<RigidBodyHandle>,
    
    pub collider_type: BodyColliderType,
    pub shape: BodyColliderShape,
}

impl BodyCollider {
    pub fn new(
        position: Vec3,
        rotation: Quat,
        collider_type: BodyColliderType,
        shape: BodyColliderShape,
    ) -> Self {
        Self {
            position,
            rotation,
            
            handle: None,

            collider_type,
            shape,
        }
    }
}

#[derive(Debug)]
pub enum BodyColliderShape {
    Box {
        size: Vec3,
    },
}

#[derive(Debug)]
pub enum BodyColliderType {
    Static,
    Kinematic,
    Dynamic,
}

impl PhysicalBodyComponent {
    pub fn new(colliders: Vec<BodyCollider>) -> Self {
        Self {
            colliders,
        }
    }
}
