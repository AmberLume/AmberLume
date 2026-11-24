use math::{Vec3, Vec4};

#[derive(Debug, Clone, Copy)]
pub struct DebugLine {
    pub start: Vec3,
    pub end: Vec3,
    pub color: Vec4,
}
