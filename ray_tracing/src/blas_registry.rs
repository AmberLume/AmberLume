use parking_lot::Mutex;
use gpu::ManagedAccelerationStructure;
use ash::vk::DeviceAddress;
use index_allocator::ResourceId;
use resource_store::GeometryRange;
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

    pub fn record_geometry(
        &self,
        id: ResourceId,
        geometry_ranges: Vec<GeometryRange>,
    ) -> Option<ManagedAccelerationStructure> {
        let entry = BlasEntry {
            geometry_ranges,
            acceleration_structure: None,
        };

        self.entries.lock()[id.inner as usize]
            .replace(entry)
            .and_then(|displaced| displaced.acceleration_structure)
    }

    pub fn geometry_ranges(&self, id: ResourceId) -> Option<Vec<GeometryRange>> {
        self.entries.lock()[id.inner as usize]
            .as_ref()
            .map(|entry| entry.geometry_ranges.clone())
    }

    pub fn set_acceleration_structure(
        &self,
        id: ResourceId,
        acceleration_structure: ManagedAccelerationStructure,
    ) -> Option<ManagedAccelerationStructure> {
        let mut entries = self.entries.lock();

        let Some(entry) = entries[id.inner as usize].as_mut() else {
            return Some(acceleration_structure);
        };

        entry.acceleration_structure.replace(acceleration_structure)
    }

    pub fn remove(&self, id: ResourceId) -> Option<ManagedAccelerationStructure> {
        self.entries.lock()[id.inner as usize]
            .take()
            .and_then(|entry| entry.acceleration_structure)
    }

    pub fn addresses(&self) -> Vec<DeviceAddress> {
        self.entries
            .lock()
            .iter()
            .map(|entry| {
                entry
                    .as_ref()
                    .and_then(|entry| entry.acceleration_structure.as_ref())
                    .map_or(0, |acceleration_structure| acceleration_structure.device_address)
            })
            .collect()
    }

    pub fn drain(&self) -> Vec<ManagedAccelerationStructure> {
        self.entries
            .lock()
            .iter_mut()
            .filter_map(|entry| entry.take().and_then(|entry| entry.acceleration_structure))
            .collect()
    }
}
