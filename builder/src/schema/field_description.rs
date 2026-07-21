use serde::{Deserialize, Serialize};
use crate::schema::field_kind::FieldKind;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldDescription {
    pub key: String,
    pub label: String,
    pub kind: FieldKind,
}

impl FieldDescription {
    pub fn new(key: &str, label: &str, kind: FieldKind) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            kind,
        }
    }
}
