use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct EnumVariant {
    pub id: u32,
    pub value: String,
    pub label: String,
}

impl EnumVariant {
    pub fn new(id: u32, value: &str, label: &str) -> Self {
        Self {
            id,
            value: value.to_string(),
            label: label.to_string(),
        }
    }
}
