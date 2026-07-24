use glam::Vec3;

#[derive(Debug, Clone)]
pub enum ColliderShape {
    Box { size: Vec3 },
    Capsule { half_height: f32, radius: f32 },
    ConvexHull { points: Vec<Vec3> },
}
