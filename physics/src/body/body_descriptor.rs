use glam::{Quat, Vec3};
use crate::body::BodyType;

#[derive(Debug, Clone)]
pub struct BodyDescriptor {
    pub body_type: BodyType,
    pub position: Vec3,
    pub rotation: Quat,
}
