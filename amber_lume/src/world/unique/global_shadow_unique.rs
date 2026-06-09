use glam::Vec3;
use shipyard::Unique;

#[derive(Unique, Debug)]
pub struct GlobalShadowUnique {
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: f32,
    pub ambient: f32,
}

impl GlobalShadowUnique {
    pub fn new() -> Self {
        Self {
            direction: Vec3::new(0.0, -1.0, 0.0),
            color: Vec3::ONE,
            intensity: 1.0,
            ambient: 0.05,
        }
    }
}
