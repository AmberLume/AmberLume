use serde::{Deserialize, Serialize};
use crate::schema::field_description::FieldDescription;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TypeDescription {
    pub name: String,
    pub label: String,
    pub fields: Vec<FieldDescription>,
}

impl TypeDescription {
    pub fn new(name: &str, label: &str, fields: Vec<FieldDescription>) -> Self {
        Self {
            name: name.to_string(),
            label: label.to_string(),
            fields,
        }
    }
}
