use std::collections::HashMap;
use parking_lot::Mutex;
use gpu::ManagedAccelerationStructure;
use ash::vk::DeviceAddress;
use index_allocator::ResourceId;
use resource_store::GeometryRange;
use crate::blas_entry::BlasEntry;

pub struct BLASRegistry {
    entries: Mutex<HashMap<ResourceId, BlasEntry>>,
}

impl BLASRegistry {
    pub(crate) fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
        }
    }

    pub fn record_geometry(
        &self,
        id: ResourceId,
        geometry_ranges: Vec<GeometryRange>,
    ) -> Option<ManagedAccelerationStructure> {
        let entry = BlasEntry {
            geometry_ranges,
            acceleration_structure: None,
        };

        self.entries
            .lock()
            .insert(id, entry)
            .and_then(|displaced| displaced.acceleration_structure)
    }

    pub fn geometry_ranges(&self, id: ResourceId) -> Option<Vec<GeometryRange>> {
        self.entries
            .lock()
            .get(&id)
            .map(|entry| entry.geometry_ranges.clone())
    }

    pub fn set_acceleration_structure(
        &self,
        id: ResourceId,
        acceleration_structure: ManagedAccelerationStructure,
    ) -> Option<ManagedAccelerationStructure> {
        let mut entries = self.entries.lock();

        let Some(entry) = entries.get_mut(&id) else {
            return Some(acceleration_structure);
        };

        entry.acceleration_structure.replace(acceleration_structure)
    }

    pub fn remove(&self, id: ResourceId) -> Option<ManagedAccelerationStructure> {
        self.entries
            .lock()
            .remove(&id)
            .and_then(|entry| entry.acceleration_structure)
    }

    pub fn addresses(&self, capacity: usize) -> Vec<DeviceAddress> {
        let mut addresses = vec![0; capacity];

        for (id, entry) in self.entries.lock().iter() {
            let Some(acceleration_structure) = &entry.acceleration_structure else {
                continue;
            };

            addresses[id.inner as usize] = acceleration_structure.device_address;
        }

        addresses
    }

    pub fn drain(&self) -> Vec<ManagedAccelerationStructure> {
        self.entries
            .lock()
            .drain()
            .filter_map(|(_, entry)| entry.acceleration_structure)
            .collect()
    }
}
