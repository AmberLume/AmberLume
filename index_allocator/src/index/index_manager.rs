use crate::index::index_manager_statistics::IndexManagerStatistics;
use crate::resource_id::ResourceId;
use parking_lot::Mutex;
use std::collections::{BTreeSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

struct RetiredIndex {
    index: ResourceId,
    ready_frame: u64,
}

pub struct IndexManager {
    inner: Mutex<IndexState>,
    capacity: u32,
    frames_in_flight: u32,
    current_frame: Arc<AtomicU64>,
}

struct IndexState {
    available: BTreeSet<ResourceId>,

    grave: VecDeque<RetiredIndex>,

    next_id: ResourceId,
}

impl IndexManager {
    pub fn new(capacity: u32, frames_in_flight: u32, current_frame: Arc<AtomicU64>) -> Self {
        let index_state = IndexState {
            available: BTreeSet::new(),
            grave: VecDeque::new(),
            next_id: ResourceId::from(0),
        };

        Self {
            inner: Mutex::new(index_state),
            capacity,
            frames_in_flight,
            current_frame,
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
        let current_frame = self.current_frame.load(Ordering::Relaxed);

        let mut inner = self.inner.lock();

        inner.grave.push_back(RetiredIndex {
            index,
            ready_frame: current_frame + self.frames_in_flight as u64,
        });
    }

    pub fn update(&self) -> Vec<ResourceId> {
        let current_frame = self.current_frame.load(Ordering::Relaxed);

        let mut inner = self.inner.lock();
        let mut freed_indices = Vec::new();

        while let Some(retired) = inner.grave.front() {
            if retired.ready_frame <= current_frame {
                let index = inner.grave.pop_front().unwrap().index;
                inner.available.insert(index);
                freed_indices.push(index);
            } else {
                break;
            }
        }

        while let Some(&last_free) = inner.available.iter().next_back() {
            if last_free.inner == inner.next_id.inner - 1 {
                inner.next_id.inner -= 1;
                inner.available.remove(&last_free);
            } else {
                break;
            }
        }

        freed_indices
    }

    pub fn statistics(&self) -> IndexManagerStatistics {
        let inner = self.inner.lock();

        let available = inner.available.len() as u32;
        let used = inner.next_id.inner - available;
        let grave = inner.grave.len() as u32;

        IndexManagerStatistics {
            capacity: self.capacity,

            used,
            free: self.capacity - inner.next_id.inner + available,
            grave,
        }
    }
}
