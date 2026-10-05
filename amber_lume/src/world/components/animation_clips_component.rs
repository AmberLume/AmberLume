use resource_residency::ResRef;
use shipyard::Component;
use std::sync::Arc;

#[derive(Component)]
pub struct AnimationClipsComponent {
    pub clips: Vec<Arc<ResRef>>,
}
