use std::sync::Arc;
use crate::resources::index::index_manager::IndexManager;
use crate::resources::store::providers::resource_provider::ResourceId;

pub struct BindlessImage {
    pub slot: ResourceId,
    index_manager: Arc<IndexManager>,
}

impl BindlessImage {
    pub fn new(
        slot: ResourceId,
        index_manager: Arc<IndexManager>,
    ) -> Self {
        Self {
            slot,
            index_manager,
        }
    }
}

impl Drop for BindlessImage {
    fn drop(&mut self) {
        self.index_manager.release(self.slot);
    }
}
