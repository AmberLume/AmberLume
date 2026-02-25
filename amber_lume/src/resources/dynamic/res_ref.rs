use crate::resources::descriptor_index_manager::IndexManager;
use crate::resources::dynamic::resource_provider::ResourceId;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

pub struct ResRef {
    pub id: ResourceId,

    pub index_manager: Arc<IndexManager>,
    pub frame_counter: Arc<AtomicU64>,
}

impl Drop for ResRef {
    fn drop(&mut self) {
        self.index_manager.release(self.id);
    }
}
