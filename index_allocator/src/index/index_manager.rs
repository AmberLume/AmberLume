use crate::index::index_manager_statistics::IndexManagerStatistics;
use crate::resource_id::ResourceId;
use parking_lot::Mutex;
use std::collections::BTreeSet;

pub struct IndexManager {
    inner: Mutex<IndexState>,
    capacity: u32,
}

struct IndexState {
    available: BTreeSet<ResourceId>,

    next_id: ResourceId,
}

impl IndexManager {
    pub fn new(capacity: u32) -> Self {
        let index_state = IndexState {
            available: BTreeSet::new(),
            next_id: ResourceId::from(0),
        };

        Self {
            inner: Mutex::new(index_state),
            capacity,
        }
    }

    pub fn acquire(&self) -> Option<ResourceId> {
        let mut inner = self.inner.lock();

        if let Some(index) = inner.available.pop_first() {
            return Some(index);
        }

        if inner.next_id.inner < self.capacity {
            let index = inner.next_id;

            inner.next_id.inner += 1;

            return Some(index);
        }

        None
    }

    pub fn release(&self, index: ResourceId) {
        let mut inner = self.inner.lock();

        inner.available.insert(index);

        while let Some(&last_free) = inner.available.iter().next_back() {
            if last_free.inner == inner.next_id.inner - 1 {
                inner.next_id.inner -= 1;
                inner.available.remove(&last_free);
            } else {
                break;
            }
        }
    }

    pub fn statistics(&self) -> IndexManagerStatistics {
        let inner = self.inner.lock();

        let available = inner.available.len() as u32;
        let used = inner.next_id.inner - available;

        IndexManagerStatistics {
            capacity: self.capacity,

            used,
            free: self.capacity - inner.next_id.inner + available,
        }
    }
}
