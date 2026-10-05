use resource_residency::ResRef;
use std::sync::Arc;

pub struct ManagedMaterial {
    pub images: Vec<Arc<ResRef>>,
}
