use rapier3d::prelude::{ColliderHandle, RigidBodyHandle};
use shipyard::Component;

#[derive(Component, Debug)]
pub struct PhysicalBodyComponent {
    pub rigid_body_handle: RigidBodyHandle,
    
    pub collider_handles: Vec<ColliderHandle>,

    pub skip_synchronization: bool,
}
