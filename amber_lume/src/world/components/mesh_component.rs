use resource_residency::ResRef;
use shipyard::Component;
use std::sync::Arc;

#[derive(Component)]
pub struct MeshComponent {
    pub handle: Arc<ResRef>,
}
