use crate::resources::store::providers::res_ref::ResRef;
use shipyard::Component;
use std::sync::Arc;

#[derive(Component)]
pub struct MeshComponent {
    pub handle: Arc<ResRef>,
    
    pub skeleton: Option<Arc<ResRef>>,
}

impl MeshComponent {
    pub fn new(
        handle: Arc<ResRef>,
        skeleton: Option<Arc<ResRef>>,
    ) -> Self {
        Self { handle, skeleton }
    }
}
