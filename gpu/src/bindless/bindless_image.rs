use std::sync::Arc;
use index_allocator::DeferredDestroy;
use index_allocator::IndexManager;
use index_allocator::ResourceId;

pub struct BindlessImage {
    pub slot: ResourceId,
    index_manager: Arc<IndexManager>,
    deferred_destroy: Arc<DeferredDestroy>,
}

impl BindlessImage {
    pub fn new(
        slot: ResourceId,
        index_manager: Arc<IndexManager>,
        deferred_destroy: Arc<DeferredDestroy>,
    ) -> Self {
        Self {
            slot,
            index_manager,
            deferred_destroy,
        }
    }
}

impl Drop for BindlessImage {
    fn drop(&mut self) {
        let index_manager = self.index_manager.clone();
        let slot = self.slot;

        self.deferred_destroy.push(move || {
            index_manager.release(slot);

            Ok(())
        });
    }
}
