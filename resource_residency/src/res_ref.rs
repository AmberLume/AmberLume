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

impl Drop for ResRef {
    fn drop(&mut self) {
        self.drop_tx.send(self.id).ok();
    }
}
