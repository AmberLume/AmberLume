use std::hash::{Hash, Hasher};
use crossbeam_channel::Sender;
use index_allocator::ResourceId;

pub struct ResRef {
    pub id: ResourceId,

    drop_tx: Sender<ResourceId>,
}

impl ResRef {
    pub(crate) fn new(id: ResourceId, drop_tx: Sender<ResourceId>) -> Self {
        Self {
            id,
            drop_tx,
        }
    }
}

impl Hash for ResRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Self {
            id,

            drop_tx: _,
        } = self;
        
        id.hash(state);
    }
}

impl Drop for ResRef {
    fn drop(&mut self) {
        self.drop_tx.send(self.id).ok();
    }
}
