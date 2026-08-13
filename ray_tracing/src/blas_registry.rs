use std::collections::HashMap;
use parking_lot::Mutex;
use crate::managed_acceleration_structure::ManagedAccelerationStructure;
use index_allocator::ResourceId;

pub struct BLASRegistry {
    entries: Mutex<HashMap<ResourceId, ManagedAccelerationStructure>>,
}

impl BLASRegistry {
    pub(crate) fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
        }
    }

    pub fn insert(
        &self,
        id: ResourceId,
        acceleration_structure: ManagedAccelerationStructure,
    ) -> Option<ManagedAccelerationStructure> {
        self.entries.lock().insert(id, acceleration_structure)
    }

    pub fn drain(&self) -> Vec<ManagedAccelerationStructure> {
        self.entries.lock().drain().map(|(_, entry)| entry).collect()
    }
}
