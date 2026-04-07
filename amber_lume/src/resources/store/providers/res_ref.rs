use std::hash::{Hash, Hasher};
use crossbeam_channel::Sender;
use crate::resources::store::providers::resource_provider::ResourceId;

pub struct ResRef {
    pub id: ResourceId,

    drop_rx: Sender<ResourceId>,
}

impl ResRef {
    pub fn new(id: ResourceId, drop_rx: Sender<ResourceId>) -> Self {
        Self {
            id,
            drop_rx,
        }
    }
}

impl Hash for ResRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Self {
            id,
            
            drop_rx: _,
        } = self;
        
        id.hash(state);
    }
}

impl Drop for ResRef {
    fn drop(&mut self) {
        self.drop_rx.send(self.id).ok();
    }
}
