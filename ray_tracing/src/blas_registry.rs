use parking_lot::Mutex;
use index_allocator::ResourceId;
use crate::blas_entry::BlasEntry;

pub struct BLASRegistry {
    entries: Mutex<Vec<Option<BlasEntry>>>,
}

impl BLASRegistry {
    pub(crate) fn new(capacity: u32) -> Self {
        Self {
            entries: Mutex::new((0..capacity).map(|_| None).collect()),
        }
    }

    pub fn insert(&self, id: ResourceId, entry: BlasEntry) -> Option<BlasEntry> {
        self.entries.lock()[id.inner as usize].replace(entry)
    }

    pub fn with_entry<R>(&self, id: ResourceId, action: impl FnOnce(&mut BlasEntry) -> R) -> Option<R> {
        self.entries.lock()[id.inner as usize].as_mut().map(action)
    }

    pub fn remove(&self, id: ResourceId) -> Option<BlasEntry> {
        self.entries.lock()[id.inner as usize].take()
    }

    pub fn drain(&self) -> Vec<BlasEntry> {
        self.entries
            .lock()
            .iter_mut()
            .filter_map(|entry| entry.take())
            .collect()
    }
}
