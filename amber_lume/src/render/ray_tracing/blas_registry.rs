use std::collections::HashMap;
use parking_lot::Mutex;
use crate::render::ray_tracing::managed_acceleration_structure::ManagedAccelerationStructure;
use crate::resources::store::providers::resource_provider::ResourceId;

pub struct BLASRegistry {
    entries: Mutex<HashMap<ResourceId, ManagedAccelerationStructure>>,
}

impl BLASRegistry {
    pub fn new() -> Self {
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
