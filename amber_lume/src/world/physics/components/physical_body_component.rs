use glam::{Quat, Vec3};
use physics::{BodyHandle, ColliderHandle};
use shipyard::Component;

#[derive(Component, Debug)]
#[track(Deletion)]
pub struct PhysicalBodyComponent {
    pub rigid_body_handle: BodyHandle,

    pub colliders: Vec<PhysicalBodyCollider>,

    pub skip_synchronization: bool,
}

#[derive(Debug)]
pub struct PhysicalBodyCollider {
    pub handle: ColliderHandle,

    pub position: Vec3,
    pub rotation: Quat,
}
