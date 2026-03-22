use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct PhysicalBodyData {
    pub name: String,

    pub colliders: Vec<ColliderData>,
}

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct ColliderData {
    pub collider_name: String,

    pub collider_shape: ColliderShape,

    pub density: f32,
    pub friction: f32,
    pub restitution: f32,

    pub translation: [f32; 3],
    pub rotation: [f32; 4],
}

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub enum ColliderShape {
    Box {
        size: [f32; 3],
    },
    ConvexHull {
        vertices: Vec<[f32; 3]>,
    },
}
