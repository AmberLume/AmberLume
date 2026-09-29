use resource_residency::ResRef;
use shipyard::Component;
use std::sync::Arc;

#[derive(Component)]
pub struct SkinComponent {
    pub handle: Arc<ResRef>,
}
