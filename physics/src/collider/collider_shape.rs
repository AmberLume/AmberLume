use glam::Vec3;

#[derive(Debug, Clone)]
pub enum ColliderShape {
    Box { size: Vec3 },
    ConvexHull { points: Vec<Vec3> },
}
