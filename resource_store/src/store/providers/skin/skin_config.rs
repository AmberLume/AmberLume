use resource_residency::ResRef;
use std::sync::Arc;

#[derive(Clone, Hash)]
pub struct SkinConfig {
    pub owner: u64,

    pub mesh: Arc<ResRef>,
    pub skeleton: Arc<ResRef>,
}
