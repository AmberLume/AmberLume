use std::fs::{create_dir_all, write};
use std::path::Path;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use crate::schema::enum_variant::EnumVariant;
use crate::schema::field_kind::FieldKind;
use crate::schema::field_description::FieldDescription;
use crate::schema::type_description::TypeDescription;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ParamSchema {
    pub types: Vec<TypeDescription>,
}

impl ParamSchema {
    pub fn amberlume() -> Self {
        Self {
            types: vec![
                TypeDescription::new("Collider", "Collider", vec![
                    FieldDescription::new("collider_shape", "Shape", FieldKind::Enum {
                        variants: vec![
                            EnumVariant::new("Box", "Box"),
                            EnumVariant::new("ConvexHull", "Convex Hull"),
                        ],
                        default: "Box".to_string(),
                    }),
                    FieldDescription::new("friction", "Friction", FieldKind::Float {
                        default: 0.5,
                        min: Some(0.0),
                        max: Some(1.0),
                        soft_min: None,
                        soft_max: None,
                    }),
                    FieldDescription::new("restitution", "Restitution", FieldKind::Float {
                        default: 0.5,
                        min: Some(0.0),
                        max: Some(1.0),
                        soft_min: None,
                        soft_max: None,
                    }),
                    FieldDescription::new("density", "Density", FieldKind::Float {
                        default: 1000.0,
                        min: Some(0.0),
                        max: None,
                        soft_min: None,
                        soft_max: Some(20000.0),
                    }),
                ]),
                TypeDescription::new("Placeholder", "Placeholder", vec![
                    FieldDescription::new("body_type", "Body Type", FieldKind::Enum {
                        variants: vec![
                            EnumVariant::new("Static", "Static"),
                            EnumVariant::new("Kinematic", "Kinematic"),
                            EnumVariant::new("Dynamic", "Dynamic"),
                        ],
                        default: "Static".to_string(),
                    }),
                ]),
                TypeDescription::new("Mesh", "Mesh", vec![]),
            ],
        }
    }

    pub fn emit(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }

        let json = to_string_pretty(self)?;
        write(path, json)?;

        Ok(())
    }
}
