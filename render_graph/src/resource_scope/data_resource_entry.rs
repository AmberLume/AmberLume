use std::any::{Any, TypeId};
use crate::virtual_data::data_origin::DataOrigin;

pub struct DataResourceEntry {
    pub label: &'static str,
    pub origin: DataOrigin,
    pub type_id: TypeId,

    pub value: Option<Box<dyn Any + Send + Sync>>,
}

impl DataResourceEntry {
    pub fn new(label: &'static str, origin: DataOrigin, type_id: TypeId) -> Self {
        Self {
            label,
            origin,
            type_id,

            value: None,
        }
    }
}
