use crate::resources::dynamic::res_ref::ResRef;
use shipyard::Component;
use std::sync::Arc;

#[derive(Component)]
pub struct MeshComponent {
    pub handle: Arc<ResRef>,
}

impl MeshComponent {
    pub fn new(handle: Arc<ResRef>) -> Self {
        Self { handle }
    }
}
