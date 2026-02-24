use std::collections::{BTreeSet, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use parking_lot::Mutex;
use crate::resources::dynamic::resource_provider::ResourceId;

struct RetiredIndex {
    index: u32,
    ready_frame: u64,
}

pub struct IndexManager {
    inner: Mutex<IndexState>,
    capacity: u32,
    frames_in_flight: u64,
    current_frame: Arc<AtomicU64>,
}

struct IndexState {
    available: BTreeSet<ResourceId>,

    grave: VecDeque<RetiredIndex>,

    next_id: ResourceId,
}

impl IndexManager {
    pub fn new(
        capacity: u32,
        frames_in_flight: u64,
        current_frame: Arc<AtomicU64>,
    ) -> Self {
        let index_state = IndexState {
            available: BTreeSet::new(),
            grave: VecDeque::new(),
            next_id: 0,
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

        if inner.next_id < self.capacity {
            let index = inner.next_id;

            inner.next_id += 1;

            return Some(index);
        }

        None
    }

    pub fn acquire_range(&self, count: u32) -> Option<ResourceId> {
        if count == 0 { return None; }
        if count == 1 { return self.acquire(); }

        let mut inner = self.inner.lock();

        if inner.next_id + count <= self.capacity {
            let start_index = inner.next_id;
            inner.next_id += count;
            return Some(start_index);
        }

        None
    }

    pub fn release(&self, index: ResourceId) {
        let current_frame = self.current_frame.load(Ordering::Relaxed);
        
        let mut inner = self.inner.lock();

        inner.grave.push_back(RetiredIndex {
            index,
            ready_frame: current_frame + self.frames_in_flight,
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
            if last_free == inner.next_id - 1 {
                inner.next_id -= 1;
                inner.available.remove(&last_free);
            } else {
                break;
            }
        }

        freed_indices
    }

    pub fn usage_stats(&self) -> (ResourceId, usize) {
        let inner = self.inner.lock();

        (inner.next_id - inner.available.len() as u32, inner.grave.len())
    }
}
