use std::collections::HashMap;
use parking_lot::Mutex;
use gpu::ManagedAccelerationStructure;
use ash::vk::DeviceAddress;
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

    pub fn contains(&self, id: ResourceId) -> bool {
        self.entries.lock().contains_key(&id)
    }

    pub fn remove(&self, id: ResourceId) -> Option<ManagedAccelerationStructure> {
        self.entries.lock().remove(&id)
    }

    pub fn addresses(&self, capacity: usize) -> Vec<DeviceAddress> {
        let mut addresses = vec![0; capacity];

        for (id, acceleration_structure) in self.entries.lock().iter() {
            addresses[id.inner as usize] = acceleration_structure.device_address;
        }

        addresses
    }

    pub fn drain(&self) -> Vec<ManagedAccelerationStructure> {
        self.entries.lock().drain().map(|(_, entry)| entry).collect()
    }
}
