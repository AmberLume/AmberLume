use glam::{Quat, Vec3};
use rapier3d::prelude::{ColliderHandle, RigidBodyHandle};
use shipyard::Component;

#[derive(Component, Debug)]
pub struct PhysicalBodyComponent {
    pub rigid_body_handle: RigidBodyHandle,
    
    pub colliders: Vec<PhysicalBodyCollider>,

    pub skip_synchronization: bool,
}

#[derive(Debug)]
pub struct PhysicalBodyCollider {
    pub handle: ColliderHandle,

    pub position: Vec3,
    pub rotation: Quat,
}
