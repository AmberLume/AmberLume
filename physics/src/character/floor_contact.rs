use glam::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct FloorContact {
    pub distance: f32,
    pub normal: Vec3,
}
