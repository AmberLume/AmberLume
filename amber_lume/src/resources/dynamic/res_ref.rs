use crate::resources::descriptor_index_manager::DescriptorIndexManager;
use crate::resources::dynamic::resource_provider::ResourceId;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct ResRef {
    pub id: ResourceId,

    pub index_manager: Arc<DescriptorIndexManager>,
    pub frame_counter: Arc<AtomicU64>,
}

impl Drop for ResRef {
    fn drop(&mut self) {
        let current_frame = self.frame_counter.load(Ordering::Acquire);

        self.index_manager.release(self.id, current_frame);
    }
}
