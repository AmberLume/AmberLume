use serde::{Deserialize, Serialize};
use crate::schema::enum_variant::EnumVariant;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FieldKind {
    Enum {
        variants: Vec<EnumVariant>,
        default: String,
    },
    Float {
        default: f32,
        min: Option<f32>,
        max: Option<f32>,
        soft_min: Option<f32>,
        soft_max: Option<f32>,
    },
    Bool {
        default: bool,
    },
    Text {
        default: String,
    },
}
