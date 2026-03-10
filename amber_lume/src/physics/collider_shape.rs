use builder::data::scene_data::BodyColliderShapeData;

#[derive(Debug, Clone)]
pub struct ColliderShape {
    pub shape_type: ShapeType,
}

#[repr(u32)]
#[derive(Debug, Clone)]
pub enum ShapeType {
    Box {
        size: [f32; 3],
    },
    Sphere {
        radius: f32,
    },
    ConvexHull {
        vertices: Vec<[f32; 3]>,
    },
}

impl ColliderShape {
    pub fn from_data(data: BodyColliderShapeData) -> Self {
        let shape_type = match data {
            BodyColliderShapeData::Box { size } => ShapeType::Box { size },
            BodyColliderShapeData::Sphere { radius } => ShapeType::Sphere { radius },
            BodyColliderShapeData::ConvexHull { vertices } => ShapeType::ConvexHull { vertices },
        };

        Self {
            shape_type,
        }
    }
}
