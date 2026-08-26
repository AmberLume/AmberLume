use crate::virtual_acceleration_structure::physical_acceleration_structure::PhysicalAccelerationStructure;
use crate::virtual_acceleration_structure::virtual_acceleration_structure::VirtualAccelerationStructure;
use ash::vk::AccelerationStructureKHR;
use std::collections::HashMap;

pub struct AccelerationStructureResourceScope {
    acceleration_structure_entries:
        HashMap<VirtualAccelerationStructure, PhysicalAccelerationStructure>,
    acceleration_structure_handles: HashMap<&'static str, VirtualAccelerationStructure>,
    next_acceleration_structure_id: u32,
}

impl AccelerationStructureResourceScope {
    pub fn new() -> Self {
        Self {
            acceleration_structure_entries: HashMap::new(),
            acceleration_structure_handles: HashMap::new(),
            next_acceleration_structure_id: 0,
        }
    }

    pub fn import_acceleration_structure(
        &mut self,
        label: &'static str,
    ) -> VirtualAccelerationStructure {
        let entry = PhysicalAccelerationStructure {
            handle: AccelerationStructureKHR::null(),
            descriptor_id: 0,
        };

        if let Some(&handle) = self.acceleration_structure_handles.get(label) {
            self.acceleration_structure_entries.insert(handle, entry);

            return handle;
        }

        let handle = VirtualAccelerationStructure::new(self.next_acceleration_structure_id);
        self.next_acceleration_structure_id += 1;

        self.acceleration_structure_handles.insert(label, handle);
        self.acceleration_structure_entries.insert(handle, entry);

        handle
    }

    pub fn rebind_acceleration_structure(
        &mut self,
        handle: VirtualAccelerationStructure,
        acceleration_structure: AccelerationStructureKHR,
        descriptor_id: u32,
    ) {
        self.acceleration_structure_entries.insert(
            handle,
            PhysicalAccelerationStructure {
                handle: acceleration_structure,
                descriptor_id,
            },
        );
    }

    pub fn get_physical_acceleration_structure(
        &self,
        handle: VirtualAccelerationStructure,
    ) -> PhysicalAccelerationStructure {
        *self
            .acceleration_structure_entries
            .get(&handle)
            .expect("Unknown VirtualAccelerationStructure handle")
    }

    pub fn is_bound(&self, handle: VirtualAccelerationStructure) -> bool {
        self.acceleration_structure_entries
            .get(&handle)
            .is_some_and(|entry| entry.handle != AccelerationStructureKHR::null())
    }
}
