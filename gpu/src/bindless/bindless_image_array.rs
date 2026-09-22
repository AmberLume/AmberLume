use std::mem::take;
use std::sync::Arc;
use index_allocator::DeferredDestroy;
use index_allocator::IndexManager;
use index_allocator::ResourceId;

pub struct BindlessImageArray {
    pub slots: Vec<ResourceId>,
    index_manager: Arc<IndexManager>,
    deferred_destroy: Arc<DeferredDestroy>,
}

impl BindlessImageArray {
    pub fn new(
        slots: Vec<ResourceId>,
        index_manager: Arc<IndexManager>,
        deferred_destroy: Arc<DeferredDestroy>,
    ) -> Self {
        Self {
            slots,
            index_manager,
            deferred_destroy,
        }
    }
}

impl Drop for BindlessImageArray {
    fn drop(&mut self) {
        let index_manager = self.index_manager.clone();
        let slots = take(&mut self.slots);

        self.deferred_destroy.push(move || {
            for slot in slots {
                index_manager.release(slot);
            }

            Ok(())
        });
    }
}
