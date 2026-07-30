use glam::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct SlideMovement {
    pub translation: Vec3,
    pub velocity: Vec3,
    pub blocked: bool,
}
