use std::sync::Arc;
use crate::resources::index::index_manager::IndexManager;
use crate::resources::store::providers::resource_provider::ResourceId;

pub struct BindlessImageArray {
    pub slots: Vec<ResourceId>,
    index_manager: Arc<IndexManager>,
}

impl BindlessImageArray {
    pub fn new(
        slots: Vec<ResourceId>,
        index_manager: Arc<IndexManager>,
    ) -> Self {
        Self {
            slots,
            index_manager,
        }
    }
}

impl Drop for BindlessImageArray {
    fn drop(&mut self) {
        for slot in self.slots.drain(..) {
            self.index_manager.release(slot);
        }
    }
}
