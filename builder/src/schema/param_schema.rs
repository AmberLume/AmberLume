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
    pub components: Vec<TypeDescription>,
}

impl ParamSchema {
    pub fn amberlume() -> Self {
        Self {
            types: vec![
                TypeDescription::new(1, "Collider", "Collider", vec![
                    FieldDescription::new("collider_shape", "Shape", FieldKind::Enum {
                        variants: vec![
                            EnumVariant::new(0, "Box", "Box"),
                            EnumVariant::new(1, "Capsule", "Capsule"),
                            EnumVariant::new(2, "ConvexHull", "Convex Hull"),
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
                TypeDescription::new(2, "Placeholder", "Placeholder", vec![
                    FieldDescription::new("body_type", "Body Type", FieldKind::Enum {
                        variants: vec![
                            EnumVariant::new(0, "Static", "Static"),
                            EnumVariant::new(1, "Kinematic", "Kinematic"),
                            EnumVariant::new(2, "Dynamic", "Dynamic"),
                        ],
                        default: "Static".to_string(),
                    }),
                ]),
                TypeDescription::new(3, "Mesh", "Mesh", vec![]),
                TypeDescription::new(4, "Skeleton", "Skeleton", vec![]),
            ],
            components: vec![
                TypeDescription::new(0, "Character", "Character", vec![
                    FieldDescription::new("offset", "Offset", FieldKind::Float {
                        default: 0.01,
                        min: Some(0.0),
                        max: None,
                        soft_min: None,
                        soft_max: Some(1.0),
                    }),
                    FieldDescription::new("auto_step_height", "Auto Step Height", FieldKind::Float {
                        default: 0.25,
                        min: Some(0.0),
                        max: None,
                        soft_min: None,
                        soft_max: Some(2.0),
                    }),
                    FieldDescription::new("max_slope_angle", "Max Slope Angle", FieldKind::Float {
                        default: 45.0,
                        min: Some(0.0),
                        max: Some(90.0),
                        soft_min: None,
                        soft_max: None,
                    }),
                    FieldDescription::new("speed", "Speed", FieldKind::Float {
                        default: 10.0,
                        min: Some(0.0),
                        max: None,
                        soft_min: None,
                        soft_max: Some(100.0),
                    }),
                    FieldDescription::new("push_force", "Push Force", FieldKind::Float {
                        default: 30.0,
                        min: Some(0.0),
                        max: None,
                        soft_min: None,
                        soft_max: Some(1000.0),
                    }),
                    FieldDescription::new("jump_velocity", "Jump Velocity", FieldKind::Float {
                        default: 10.0,
                        min: Some(0.0),
                        max: None,
                        soft_min: None,
                        soft_max: Some(100.0),
                    }),
                ]),
                TypeDescription::new(1, "Camera", "Camera", vec![
                    FieldDescription::new("fov", "FOV", FieldKind::Float {
                        default: 80.0,
                        min: Some(1.0),
                        max: Some(179.0),
                        soft_min: None,
                        soft_max: None,
                    }),
                    FieldDescription::new("near", "Near", FieldKind::Float {
                        default: 0.3,
                        min: Some(0.001),
                        max: None,
                        soft_min: None,
                        soft_max: Some(10.0),
                    }),
                    FieldDescription::new("far", "Far", FieldKind::Float {
                        default: 10000.0,
                        min: Some(0.01),
                        max: None,
                        soft_min: None,
                        soft_max: Some(100000.0),
                    }),
                ]),
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
