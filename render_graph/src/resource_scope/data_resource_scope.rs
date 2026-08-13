use std::any::{type_name, TypeId};
use std::collections::HashMap;
use crate::resource_scope::data_resource_entry::DataResourceEntry;
use crate::virtual_data::data_key::DataKey;
use crate::virtual_data::virtual_data::VirtualData;

pub struct DataResourceScope {
    entries: Vec<DataResourceEntry>,
    handles: HashMap<&'static str, DataKey>,
}

impl DataResourceScope {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            handles: HashMap::new(),
        }
    }

    pub fn import_data<T: Send + Sync + 'static>(&mut self, label: &'static str) -> VirtualData<T> {
        let type_id = TypeId::of::<T>();

        if let Some(&key) = self.handles.get(label) {
            let entry = &self.entries[key.handle as usize];

            assert_eq!(
                entry.type_id, type_id,
                "Data label '{label}' is already declared with a different type",
            );

            return VirtualData::new(key);
        }

        let key = DataKey::new(self.entries.len() as u32);

        self.entries.push(DataResourceEntry::new(label, type_id));
        self.handles.insert(label, key);

        VirtualData::new(key)
    }

    pub fn set<T: Send + Sync + 'static>(&mut self, data: VirtualData<T>, value: T) {
        self.entries[data.key.handle as usize].value = Some(Box::new(value));
    }

    pub fn get<T: Send + Sync + 'static>(&self, data: VirtualData<T>) -> &T {
        let entry = &self.entries[data.key.handle as usize];

        let Some(value) = entry.value.as_ref() else {
            panic!("Data '{}' of type {} was not imported", entry.label, type_name::<T>());
        };

        value.downcast_ref::<T>().expect("Data type mismatch")
    }

    pub fn is_available(&self, key: DataKey) -> bool {
        self.entries[key.handle as usize].value.is_some()
    }

    pub fn clear_frame(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.value = None;
        }
    }
}
