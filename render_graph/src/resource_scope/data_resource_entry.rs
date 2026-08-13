use std::any::{Any, TypeId};

pub struct DataResourceEntry {
    pub label: &'static str,
    pub type_id: TypeId,

    pub value: Option<Box<dyn Any + Send + Sync>>,
}

impl DataResourceEntry {
    pub fn new(label: &'static str, type_id: TypeId) -> Self {
        Self {
            label,
            type_id,

            value: None,
        }
    }
}
